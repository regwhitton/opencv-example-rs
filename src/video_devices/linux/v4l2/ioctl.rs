//! The ioctl functions for the v4l2 module and the structs they use.

pub const V4L2_CAP_VIDEO_CAPTURE: u32 = 0x00000001;
pub const V4L2_CAP_VIDEO_CAPTURE_MPLANE: u32 = 0x00001000;
pub const V4L2_BUF_TYPE_VIDEO_CAPTURE: u32 = 1;
pub const V4L2_BUF_TYPE_VIDEO_CAPTURE_MPLANE: u32 = 9;
pub const V4L2_FRMSIZE_TYPE_DISCRETE: u32 = 1;
pub const V4L2_FRMSIZE_TYPE_CONTINUOUS: u32 = 2;
pub const V4L2_FRMSIZE_TYPE_STEPWISE: u32 = 3;

const VIDIOC: u8 = b'V';
const VIDIOC_QUERYCAP: u8 = 0;
const VIDIOC_ENUM_FMT: u8 = 2;
const VIDIOC_ENUM_FRAMESIZES: u8 = 74;

// https://www.kernel.org/doc/html/latest/userspace-api/media/v4l/vidioc-querycap.html

#[repr(C)]
pub struct V4l2Capability {
    pub driver: [u8; 16],
    pub card: [u8; 32],
    pub bus_info: [u8; 32],
    pub version: u32,
    pub capabilities: u32,
    pub device_caps: u32,
    pub reserved: [u32; 3]
}

nix::ioctl_read!(vidioc_querycap, VIDIOC, VIDIOC_QUERYCAP, V4l2Capability);

// https://www.kernel.org/doc/html/latest/userspace-api/media/v4l/vidioc-enum-fmt.html

#[repr(C)]
pub struct V4l2Fmtdesc {
    pub index: u32,
    pub typ: u32,
    pub flags: u32,
    pub description: [u8; 32],
    pub pixel_format: u32,
    pub mbus_code: u32,
    pub reserved: [u32; 3]
}

nix::ioctl_readwrite!(vidioc_enum_fmt, VIDIOC, VIDIOC_ENUM_FMT, V4l2Fmtdesc);

// https://www.kernel.org/doc/html/latest/userspace-api/media/v4l/vidioc-enum-framesizes.html
 
#[repr(C)]
#[derive(Copy, Clone)]
pub struct V4l2FrmsizeDiscrete {
    pub width: u32,
    pub height: u32
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct V4l2FrmsizeStepwise {
    pub min_width: u32,
    pub max_width: u32,
    pub step_width: u32,
    pub min_height: u32,
    pub max_height: u32,
    pub step_height: u32
}

#[repr(C)]
pub union V4l2Frmsize {
    pub discrete: V4l2FrmsizeDiscrete,
    pub stepwise: V4l2FrmsizeStepwise
}

#[repr(C)]
pub struct V4l2Frmsizeenum {
    pub index: u32,
    pub pixel_format: u32,
    pub typ: u32,
    pub frmsize: V4l2Frmsize, 
    pub reserved: [u32; 2]
}

nix::ioctl_readwrite!(vidioc_enum_framesizes, VIDIOC, VIDIOC_ENUM_FRAMESIZES, V4l2Frmsizeenum);