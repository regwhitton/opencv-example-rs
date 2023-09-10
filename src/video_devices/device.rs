use std::vec::Vec;
use std::cmp::PartialEq;

pub struct Device {
    pub opencv_id: i32,
    pub unique_id: String,
    pub name: String,
    pub frame_size_types: Vec<FrameSizeType>
}

#[derive(PartialEq)]
#[derive(Debug)]
pub enum FrameSizeType {
    Discrete {
        width: u32,
        height: u32
    },
    Stepwise {
        min_width: u32,
        max_width: u32,
        step_width: u32,
        min_height: u32,
        max_height: u32,
        step_height: u32
    }
}

pub fn sort_devices(devices: &mut Vec<Device>) {
    devices.sort_by(
        |d1, d2| d1.unique_id
            .cmp(&d2.unique_id)
        );
}

pub fn sort_frame_size_types(frame_size_types: &mut Vec<FrameSizeType>) {
    frame_size_types.sort_by(
        |fst1, fst2| frame_area(fst1)
            .cmp(&frame_area(fst2))
            .reverse()
        );
}

fn frame_area(fst: &FrameSizeType) -> u32 {
    match fst {
        FrameSizeType::Discrete{width, height} => width * height,
        FrameSizeType::Stepwise{max_width, max_height, ..} => max_width * max_height
    }
}
