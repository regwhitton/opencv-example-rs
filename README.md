# opencv-example-rs

An example of using OpenCV from Rust.

Originally I wanted to capture images from a webcam and use them in animations.  OpenCV seems to offer a good way to get to operate lots of different webcams and cameras.  However, it doesn't give any way to list the attached cameras and what image sizes they support.

Here I have written code to list the cameras and supported sizes (Linux and Windows only), and use this to capture a frame from each camera at each supported frame size.

The docker images try to build the complete OpenCV libraries (static without FFMpeg), so that I can link a minimal executable.  I haven't used much of the functionality and can't say how much of it will work.

This uses:

* [OpenCV](https://github.com/opencv/opencv) of course.
* [opencv-rust](https://github.com/twistedfall/opencv-rust) from twistedfall.
* [cross-rs](https://github.com/cross-rs/cross) to cross compile.

## Linux

Install OpenCV:

    sudo apt-get install libopencv-dev

Build with cargo:

    cargo build
`
## Raspberry Pi

This works for USB cameras on my 2011 Raspberry Pi 1, and I believe it will also work for Pi Zero (also ARMv6).

So far it will not work with my PiCam Module 3.  I believe that opening the camera with picamera library would work, but haven't tried it.

Under desktop Linux, create the cross-rs docker image:

    make rpi01-docker

Build the example:

    make rpi01

Copy target/arm-unknown-linux-gnueabihf/debug/opencv-example-rs to your Pi and run it.

## Windows

This has built under Windows using the libraries downloaded from OpenCV.  I don't remember the details.

*I haven't yet managed to build the Windows executable under Linux, the intention is that:*

Under desktop Linux, create the cross-rs docker image:

    make winpc-docker

Build the example:

    make winpc

