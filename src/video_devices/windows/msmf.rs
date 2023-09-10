/*
 * This module finds details of Windows MSMF capture devices.
 * See:
 * https://learn.microsoft.com/en-us/windows/win32/medfound/enumerating-video-capture-devices
 * https://github.com/opencv/opencv/blob/4.x/modules/videoio/src/cap_msmf.cpp
 * Use info on using the Rust Windows crate: https://kennykerr.ca/index.html
 */
use super::Device;
use super::FrameSizeType;
use anyhow::{
    Result,
    Error
};
use windows::{
    Win32::{
        Media::MediaFoundation::{
            IMFAttributes,
            IMFActivate,
            IMFMediaSource,
            IMFPresentationDescriptor,
            IMFStreamDescriptor,
            IMFMediaTypeHandler,
            IMFMediaType,
            MFCreateAttributes,
            MFEnumDeviceSources,
            MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE,
            MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID,
            MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME,
            MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK,
            MF_MT_FRAME_SIZE
        },
        Foundation::{
            BOOL,
            TRUE
        },
        System::WinRT::{
            RoInitialize,
            RoUninitialize,
            RO_INIT_SINGLETHREADED
        }
    },
    core::{
        PWSTR,
        GUID
    }
};
use std::slice;
use opencv::videoio::CAP_MSMF;
use super::super::device::sort_frame_size_types;

pub fn add_capture_devices(devices: &mut Vec<Device>) -> Result<()> {
    using_com_thread(|| add_capture_devices_on_com_thread(devices))
}

/// Calls [RoInitialize](https://learn.microsoft.com/en-us/windows/win32/api/roapi/nf-roapi-roinitialize)
/// before executing the function, and
/// [RoUninitialize](https://learn.microsoft.com/en-us/windows/win32/api/roapi/nf-roapi-rouninitialize) after.
fn using_com_thread<F,T>(mut func: F) -> Result<T> where F: FnMut() -> Result<T> {
    // There is just too much guff around COM to be certain that I am doing the right thing.
    // Should probably init once at startup
    // see: https://github.com/microsoft/windows-rs/issues/1169
    // https://kennykerr.ca/rust-getting-started/windows-or-windows-sys.html
    
    unsafe { RoInitialize(RO_INIT_SINGLETHREADED) }?;
    let result: Result<T> = func();
    unsafe { RoUninitialize() };
    result
}

fn add_capture_devices_on_com_thread(devices: &mut Vec<Device>) -> Result<()> {
    let attrs: IMFAttributes = mf_create_attributes(
        &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE,
        &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID)?;

    let opt_device_refs: &[Option<IMFActivate>] = mf_enum_device_sources(&attrs)?;

    let mut device_index = 0;
    for opt_device_ref in opt_device_refs {
        if let Some(device_ref) = opt_device_ref {
            let device = build_device_using_ref(device_index, device_ref)?;
            devices.push(device);
        }
        device_index += 1;
    }
    return Ok(());
}

fn mf_create_attributes(guidkey: *const GUID, guidvalue: *const GUID) -> Result<IMFAttributes> {
    let mut opt_attribute_store: Option<IMFAttributes> = None;
    unsafe {
        MFCreateAttributes(&mut opt_attribute_store, 1)?;
        let attr: IMFAttributes = opt_attribute_store
            .ok_or(Error::msg("Failed to create attribute store"))?;
        attr.SetGUID(guidkey, guidvalue)?;
        Ok(attr)
    }
}

fn mf_enum_device_sources<'a>(attrs: &IMFAttributes) -> Result<&'a[Option<IMFActivate>]> {
    let mut device_sources: *mut Option<IMFActivate> = &mut None;
    let mut device_count: u32 = 0;
    unsafe {
        MFEnumDeviceSources(attrs, &mut device_sources, &mut device_count)?;
        Ok(slice::from_raw_parts(device_sources, device_count as usize))
    }
}

fn build_device_using_ref(device_index: i32, device_ref: &IMFActivate) -> Result<Device> {
    let friendly_name: String = get_device_friendly_name(device_ref)?;
    let symbolic_link: String = get_device_symbolic_link(device_ref)?;
    let frame_size_types: Vec<FrameSizeType> = find_frame_size_types(device_ref)?;

    Ok(Device{
        opencv_id: device_index | CAP_MSMF,
        unique_id: symbolic_link,
        name: friendly_name,
        frame_size_types: frame_size_types
    })
}

fn get_device_friendly_name(device_ref: &IMFActivate) -> Result<String> {
    mf_device_get_allocated_string(device_ref, &MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME)
}

fn get_device_symbolic_link(device_ref: &IMFActivate) -> Result<String> {
    mf_device_get_allocated_string(device_ref, &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK)
}

fn mf_device_get_allocated_string(device: &IMFActivate, guidkey: *const GUID) -> Result<String> {
    let mut value: PWSTR = PWSTR::null();
    let mut len: u32 = 0;
    unsafe {
        device.GetAllocatedString(guidkey, &mut value, &mut len)?;
        Ok(value.to_string()?)
    }
}

fn find_frame_size_types(device_ref: &IMFActivate) -> Result<Vec::<FrameSizeType>> {
    with_activated_media_source(device_ref,
        |media_source: &IMFMediaSource| find_frame_sizes_using_source(media_source))
}

/// Activates the device to get a media source, which it passes to the function and shuts down afterwards.
fn with_activated_media_source<F,T>(device_ref: &IMFActivate, func: F) -> Result<T> where
    F: Fn(&IMFMediaSource) -> Result<T>
{
    let media_source: IMFMediaSource = unsafe { device_ref.ActivateObject() }?;
    let result: Result<T> = func(&media_source);
    unsafe { media_source.Shutdown() }.ok();
    result
}


fn find_frame_sizes_using_source(media_source: &IMFMediaSource) -> Result<Vec::<FrameSizeType>> {
    let descriptor: IMFPresentationDescriptor = unsafe { media_source.CreatePresentationDescriptor() }?;
    let desc_count: u32 = unsafe { descriptor.GetStreamDescriptorCount() }?;

    let mut types = Vec::<FrameSizeType>::new();
    for idx in 0..desc_count {
        if let Some(stream) = get_stream_descriptor_by_idx(&descriptor, idx)? {
            add_frame_size_types_for_stream(&stream, &mut types)?;
        }
    }
    sort_frame_size_types(&mut types);
    Ok(types)
}

fn get_stream_descriptor_by_idx(descriptor: &IMFPresentationDescriptor, idx: u32) -> Result<Option<IMFStreamDescriptor>> {
    let mut pf_selected: BOOL = TRUE;
    let mut opt_desc: Option<IMFStreamDescriptor> = None;
    unsafe { descriptor.GetStreamDescriptorByIndex(idx, &mut pf_selected, &mut opt_desc) }?;

    Ok(opt_desc)
}

fn add_frame_size_types_for_stream(stream_descriptor: &IMFStreamDescriptor, types: &mut Vec::<FrameSizeType>) -> Result<()> {
    let media_type_handler: IMFMediaTypeHandler = unsafe { stream_descriptor.GetMediaTypeHandler() }?;
    let media_type_count: u32 = unsafe { media_type_handler.GetMediaTypeCount() }?;

    for idx in 0..media_type_count {
        let t = get_media_type_size_by_idx(&media_type_handler, idx)?;
        if !types.contains(&t) {
            types.push(t);
        }
    }
    Ok(())
}

fn get_media_type_size_by_idx(media_type_handler: &IMFMediaTypeHandler, idx: u32) -> Result<FrameSizeType> {
    let media_type: IMFMediaType = unsafe { media_type_handler.GetMediaTypeByIndex(idx) }?;        
    let fs = unsafe { media_type.GetUINT64(&MF_MT_FRAME_SIZE) }?;
    let height = fs as u32;
    let width = (fs >> 32) as u32;
    Ok(FrameSizeType::Discrete { width, height })
}
