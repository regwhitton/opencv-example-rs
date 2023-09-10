use anyhow::Result;
use std::vec::Vec;
use super::Device;
use super::FrameSizeType;
use super::device::sort_devices;

mod msmf;

pub fn find_capture_devices() -> Result<Vec<Device>> {
    let mut devices = Vec::<Device>::new();

    msmf::add_capture_devices(&mut devices)?;

    // Could also add UEYE devices.

    sort_devices(&mut devices);
    Ok(devices)
}
