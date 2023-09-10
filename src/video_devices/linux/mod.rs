use anyhow::Result;
use std::vec::Vec;
use super::Device;
use super::FrameSizeType;
use super::device::sort_devices;

mod v4l2;

pub fn find_capture_devices() -> Result<Vec<Device>> {
    let mut devices = Vec::<Device>::new();

    v4l2::add_capture_devices(&mut devices)?;

    // gphoto2 - would be good for the supported cameras
    // see http://gphoto.org/proj/libgphoto2/support.php
 
    // Also firewire might be good for video cameras.

    sort_devices(&mut devices);
    Ok(devices)
}