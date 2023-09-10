 //! Finds details of v4l (Video for Linux) capture devices.
 /*
 * The index accepted by cv:VideoCapture is the number in the device filename
 * (/dev/video0 to /dev/video63), but not all these files represent video capture devices.
 * See https://github.com/opencv/opencv/blob/4.x/modules/videoio/src/cap_v4l.cpp
 *
 * Devices with multiple camera heads (such as MS Kinect) produce frame grabs with
 * multiple images, so don't appear as separate devices.
 *
 * For information about the API being used see:
 * https://docs.kernel.org/userspace-api/media/v4l/v4l2.html
 * https://docs.kernel.org/userspace-api/media/v4l/dev-capture.html
 * https://www.kernel.org/doc/html/latest/userspace-api/media/v4l/dev-capture.html
 * https://www.kernel.org/doc/html/latest/userspace-api/media/v4l/user-func.html
 */
use anyhow::{
    Result,
    Error
};
use nix::errno::Errno;
use std::{
    mem,
    path::PathBuf,
    fs::{
        read_dir,
        ReadDir,
        DirEntry,
        File
    },
    os::fd::{
        IntoRawFd,
        RawFd
    },
    ffi::CStr,
    vec::Vec,
    io::ErrorKind
};
use opencv::videoio::CAP_V4L;
use super::{
    Device,
    FrameSizeType
};
use super::super::device::sort_frame_size_types;

mod ioctl;
use ioctl::*;

// pub fn add_capture_devices(devices: &mut Vec<Device>) -> Result<()> {
//     for dir_entry_result in read_dir("/dev/v4l/by-id")? {
//         let dir_entry = dir_entry_result?;
//         if is_symlink(&dir_entry) {
//             if let Some(device) = to_capture_device(&dir_entry) {
//                 devices.push(device);
//             }
//         }
//     }
//     Ok(())
// }

pub fn add_capture_devices(devices: &mut Vec<Device>) -> Result<()> {
    // by-id gives us an id that should be consistent, even when
    // device plugged into another port (say USB).
    match read_dir("/dev/v4l/by-id") {
        Ok(dir) => add_capture_devices_for_dir(devices, dir)?,
        Err(err) if err.kind() == ErrorKind::NotFound => {},
        Err(err) => { return Err(err.into()); }
    };
    // Raspberry Pi doesn't create or populate by-id for PiCam, so we need to
    // add any extra found in by-path.
    match read_dir("/dev/v4l/by-path") {
        Ok(dir) => add_capture_devices_for_dir(devices, dir)?,
        Err(err) if err.kind() == ErrorKind::NotFound => {},
        Err(err) => { return Err(err.into()); }
    };
    Ok(())
}

fn add_capture_devices_for_dir(devices: &mut Vec<Device>, dir: ReadDir) -> Result<()> {
    for dir_entry_result in dir {
        let dir_entry = dir_entry_result?;
        if is_symlink(&dir_entry) {
            let symlink_path: PathBuf = dir_entry.path();
            let device_filepath: String = to_device_filepath(&symlink_path);
            if device_is_not_list(devices, device_filepath) {
                if let Some(device) = to_capture_device(&symlink_path) {
                    devices.push(device);
                }
            }
        }
    }
    Ok(())
}

fn is_symlink(dir_entry: &DirEntry) -> bool {
    let ft = dir_entry.file_type();
    ft.is_ok() && ft.unwrap().is_symlink()
}

fn device_is_not_list(devices: &Vec<Device>, device_filepath: String) -> bool {
    let opencv_id = to_opencv_id(&device_filepath);
    !devices.iter().any(|d| d.opencv_id == opencv_id)
}

fn to_capture_device(symlink_path: &PathBuf) -> Option<Device> {
    let device_filepath: String = to_device_filepath(&symlink_path);

    match open_device_fd(&device_filepath) {
        Ok(device_fd) => {
            if let Ok(capabilities) = query_capabilities(&device_fd) {
                if is_capture_device(&capabilities) {
                        if let Ok(frame_size_types) = find_frame_size_types(&device_fd) {
                        return Some(Device{
                            opencv_id: to_opencv_id(&device_filepath),
                            unique_id: to_unique_id(&symlink_path),
                            name: to_device_name(&capabilities),
                            frame_size_types
                        });
                    }
                }
            }
            None
        },
        // Perhaps it is locked, or we don't have access.
        // We just won't list it, but we should be a bit more selective.
        Err(_) => None
    }
}

fn to_unique_id(entry_path: &PathBuf) -> String {
    let id_osstr = entry_path.file_name()
        .expect("/dev/v4l/by-id symlinks do not end in '..'");
    id_osstr.to_string_lossy().to_string()
}

fn to_device_filepath(symlink_path: &PathBuf) -> String {
    // device_path will be from "/dev/video0" to "/dev/video63",
    // depending on device probe response order.
    let resolved_path = symlink_path.canonicalize()
        .expect("/dev/v4l/by-id or /dev/v4l/by-path symlinks resolve to /dev/video paths");
    let path_str = resolved_path.to_str()
        .expect("/dev/v4l/by-id or /dev/v4l/by-path  symlinks resolve to utf8 paths");
    path_str.to_string()
}

fn to_opencv_id(device_filepath: &String) -> i32 {
    // Integer id following "/dev/video" in devive_path.
    // This is the id expected by OpenCV when opening device.
    let index_of_o = device_filepath.rfind('o')
        .expect("device paths will be /dev/video0 to /dev/video63");
    let dev_video_int = device_filepath[index_of_o+1 ..].parse::<i32>()
        .expect("/dev/video paths to end in an integer");
    dev_video_int | CAP_V4L
}

fn open_device_fd(device_filepath: &String) -> Result<RawFd> {
    let file = File::options()
        .read(true)
        .write(true)
        .open(device_filepath)?;
    Ok(file.into_raw_fd())
}

fn query_capabilities(fd: &RawFd) -> Result<ioctl::V4l2Capability> {
    unsafe {
        let mut cap: V4l2Capability = mem::zeroed();
        vidioc_querycap(fd.clone(), &mut cap)?;
        Ok(cap)
    }
}

fn is_capture_device(cap: &V4l2Capability) -> bool {
    (cap.device_caps & ( V4L2_CAP_VIDEO_CAPTURE | V4L2_CAP_VIDEO_CAPTURE_MPLANE )) != 0
}

fn to_device_name(cap: &V4l2Capability) -> String {
    if let Ok(name) = CStr::from_bytes_until_nul(&cap.card) {
        return name.to_string_lossy().to_string();
    }
    String::from("UNKNOWN")
}

fn find_frame_size_types(fd: &RawFd) -> Result<Vec<FrameSizeType>> {
    let mut types = Vec::<FrameSizeType>::new();

    add_frame_size_types(&mut types, fd, V4L2_BUF_TYPE_VIDEO_CAPTURE)?;
    add_frame_size_types(&mut types, fd, V4L2_BUF_TYPE_VIDEO_CAPTURE_MPLANE)?;
    
    sort_frame_size_types(&mut types);
    Ok(types)
}

fn add_frame_size_types(types: &mut Vec<FrameSizeType>, fd: &RawFd, fmt_type: u32) -> Result<()> {
    for format_index in 0.. {
        match query_format(fd.clone(), fmt_type, format_index) {
            Ok(fmt) => {
                for t in find_frame_size_types_for_format(fd, fmt.pixel_format)? {
                    if !types.contains(&t) {
                        types.push(t);
                    }
                }
            },
            // EINVAL returned after last index
            Err(Errno::EINVAL) => break,
            Err(errno) => Err(errno)?
        }
    }
    Ok(())
}

fn query_format(fd: RawFd, fmt_type: u32, index: u32) -> Result<V4l2Fmtdesc,Errno> {
    unsafe {
        let mut fmt: V4l2Fmtdesc = mem::zeroed();
        fmt.index = index;
        fmt.typ = fmt_type;
        vidioc_enum_fmt(fd, &mut fmt)?;
        Ok(fmt)
    }
}

fn find_frame_size_types_for_format(fd: &RawFd, pixel_format: u32) -> Result<Vec<FrameSizeType>> {
    let mut types = Vec::<FrameSizeType>::new();

    for frame_size_index in 0.. {
        match query_frame_sizes(fd.clone(), pixel_format, frame_size_index) {
            Ok(fsz)  => types.push(to_frame_size_type(&fsz)?),
            // EINVAL returned after last index
            Err(Errno::EINVAL) => break,
            Err(errno) => Err(errno)?
        }
    }
    Ok(types)
}

fn to_frame_size_type(fsz: &V4l2Frmsizeenum) -> Result<FrameSizeType> {
    match fsz.typ {
        V4L2_FRMSIZE_TYPE_DISCRETE => Ok(unsafe{
            FrameSizeType::Discrete {
                width: fsz.frmsize.discrete.width,
                height: fsz.frmsize.discrete.height
            }}),
        V4L2_FRMSIZE_TYPE_STEPWISE | V4L2_FRMSIZE_TYPE_CONTINUOUS => Ok(unsafe {
            FrameSizeType::Stepwise {
                min_width: fsz.frmsize.stepwise.min_width,
                max_width: fsz.frmsize.stepwise.max_width,
                step_width: fsz.frmsize.stepwise.step_width,
                min_height: fsz.frmsize.stepwise.min_height,
                max_height: fsz.frmsize.stepwise.max_height,
                step_height: fsz.frmsize.stepwise.step_height
            }}),
        _ => Err(Error::msg("Unknown frame size type"))
    }
}

fn query_frame_sizes(fd: RawFd, pixel_format:u32, index: u32) -> Result<V4l2Frmsizeenum,Errno> {
    unsafe {
        let mut frmsize: V4l2Frmsizeenum = mem::zeroed();
        frmsize.pixel_format = pixel_format;
        frmsize.index = index;
        vidioc_enum_framesizes(fd, &mut frmsize)?;
        Ok(frmsize)
    }
}