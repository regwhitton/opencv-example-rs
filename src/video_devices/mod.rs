#[cfg_attr(target_os = "linux", path = "linux/mod.rs")]
#[cfg_attr(target_os = "windows", path = "windows/mod.rs")]
mod os;
pub use os::find_capture_devices;

mod device;
pub use device::Device;
pub use device::FrameSizeType;