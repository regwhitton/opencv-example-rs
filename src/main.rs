mod video_devices;

use anyhow::Result; // Automatically handle the error types
use opencv::{
    //prelude::*,
    videoio::{
        get_camera_backends,
        get_backend_name,
        VideoCapture,
        CAP_ANY,
        CAP_PROP_FRAME_WIDTH,
        CAP_PROP_FRAME_HEIGHT,
        CAP_PROP_FOURCC,
        // CAP_PROP_XI_TIMEOUT,
        // CAP_PROP_OPEN_TIMEOUT_MSEC,
        // CAP_PROP_READ_TIMEOUT_MSEC,
        VideoWriter,
        // CAP_PROP_FPS
    },
    prelude::VideoCaptureTrait,
    imgcodecs::{
        imwrite,
        ImwriteFlags
    },
    core::{
        Mat,
        Vector
    }
}; // Note, the namespace of OpenCV is changed (to better or worse). It is no longer one enormous.
use video_devices::{
    find_capture_devices,
    FrameSizeType::{*, self}
};
use regex::Regex;
use std::{
    thread::sleep,
    time::Duration
};

fn main() -> Result<()> { // Note, this is anyhow::Result
    let apis = get_camera_backends()?;
    for be in &apis {
        let name = get_backend_name(be)?;
        println!("name: {}", name);
    }

    for device in find_capture_devices()? {
        println!("opencv id: {:?}", device.opencv_id);
        println!("device unique id: {:?}", device.unique_id);
        println!("device name: {:?}", device.name);
        for fz in device.frame_size_types {
            println!("  fsz: {:?}", fz);
            take_snaps(device.opencv_id, fz, &device.name)?;
        }
    }
    Ok(())
}

fn take_snaps(opencv_id: i32, fz: FrameSizeType, device_name: &String) -> Result<()> {
    match fz {
        Discrete { width, height } => {
            take_snap(opencv_id, width, height, device_name)?;
        },
        Stepwise { max_width, max_height, min_width, min_height, .. } => {
            take_snap(opencv_id, max_width, max_height, device_name)?;
            take_snap(opencv_id, min_width, min_height, device_name)?;
        }
    };
    Ok(())
}

fn take_snap(opencv_id: i32, width: u32, height: u32, device_name: &String) -> Result<()> {

    let mut prefs = Vector::<i32>::new();
    prefs.push(CAP_PROP_FRAME_WIDTH);
    prefs.push(640);
    prefs.push(CAP_PROP_FRAME_HEIGHT);
    prefs.push(480);
    // v4l2-ctl --list-formats
    prefs.push(CAP_PROP_FOURCC);
    prefs.push(VideoWriter::fourcc('B', 'G', 'R', '3')?);

    let mut vc: VideoCapture = VideoCapture::new_with_params(opencv_id, CAP_ANY, &prefs)?;
    sleep(Duration::from_millis(100));
    
    // let mut vc: VideoCapture = VideoCapture::default()?;
    // vc.open_with_params(opencv_id, CAP_ANY, &prefs)?;

    //vc.set(CAP_PROP_FRAME_WIDTH, width as f64)?;
    //vc.set(CAP_PROP_FRAME_HEIGHT, height as f64)?;
    // vc.set(CAP_PROP_FRAME_WIDTH, 640 as f64)?;
    // vc.set(CAP_PROP_FRAME_HEIGHT, 480 as f64)?;
    // vc.set(CAP_PROP_OPEN_TIMEOUT_MSEC, 40000 as f64)?;
    // vc.set(CAP_PROP_READ_TIMEOUT_MSEC, 40000 as f64)?;
    // vc.set(CAP_PROP_FPS, 30 as f64)?;
    // vc.set(CAP_PROP_FOURCC, VideoWriter::fourcc('U', 'Y', 'V', 'Y')? as f64)?;

    // let mut cameras = Vector::<VideoCapture>::new();
    // cameras.push(vc);
    // let mut ready_index = Vector::<i32>::from_elem(0, 1);
    // if !VideoCapture::wait_any(&cameras, &mut ready_index, 40000000)? {
    //     cameras.get(0)?.grab()?;
    //     let mut frame = Mat::default();
    //     cameras.get(0)?.retrieve(&mut frame, ready_index.get(0)?)?;
    
    //     let filename = to_filename(device_name, width, height)?;
    //     let mut params = Vector::<i32>::new();
    //     params.push(ImwriteFlags::IMWRITE_JPEG_QUALITY as i32);
    //     params.push(100);

    //     let _ = imwrite(filename.as_str(), &frame, &params)?;
    // }
    // cameras.get(0)?.release()?;
    // Ok(())

    let mut frame = Mat::default();
    for _ in 1..=10 {
        if vc.read(&mut frame)? {
            let filename = to_filename(device_name, width, height)?;
            let mut params = Vector::<i32>::new();
            params.push(ImwriteFlags::IMWRITE_JPEG_QUALITY as i32);
            params.push(100);

            let _ = imwrite(filename.as_str(), &frame, &params)?;
            break;
        }
        sleep(Duration::from_secs(1));
    }
    vc.release()?;
    Ok(())
}

fn to_filename(device_name: &String, width: u32, height: u32) -> Result<String> {
    let cleaning_re = Regex::new(r"[^a-zA-Z0-9_.-]")?;
    let cleaned_name = cleaning_re.replace_all(device_name.as_str(), "")
        .into_owned();
    let name = format!("img-{}-{}x{}.jpg", cleaned_name, width, height);
    Ok(name)
}


