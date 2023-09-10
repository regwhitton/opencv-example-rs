# Problems building for 2011 Raspberry Pi 1 (and Pi 0)
# The parent image compiles valid armv6 code, but some of the gcc libraries
# include armv7 instructions as do some libraries you might try to
# install from the Debian repo.  When run such code on 32 bit Raspberry Pi 1
# you see the message "Illegal Instructions".
# This downloads and uses the libraries from the Raspberry Pi distribution.
FROM ghcr.io/cross-rs/arm-unknown-linux-gnueabihf:0.2.5

# useful in case anything wants an interactive prompt
ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get install --assume-yes --no-install-recommends \
        xz-utils \
        cmake \
        unzip \
        python3 && \
    apt-get update

RUN curl -L https://downloads.raspberrypi.org/raspios_lite_armhf/root.tar.xz > /root.tar.xz

# Unpack Raspberry Pi libraries for inclusion in sysroot
RUN mkdir /rpi-root && \
    cd /rpi-root && \
    tar xf /root.tar.xz \
        ./lib \
        ./usr/lib && \
    rm /root.tar.xz

# Overwrite problem libatomic.a with one that only contains v6 instructions
RUN cp /rpi-root/usr/lib/gcc/arm-linux-gnueabihf/10/libatomic.a ${CROSS_SYSROOT}/lib/libatomic.a && \
    chown crosstool:crosstool ${CROSS_SYSROOT}/lib/libatomic.a && \
    chmod 0444 ${CROSS_SYSROOT}/lib/libatomic.a 

# GStreamer - not getting picked up and may be not required.
#RUN cd /rpi-root/usr/lib/arm-linux-gnueabihf && \
#    for f in libgst*-1.0.so.0 ;do \
#        cp $f.1804.0 ${CROSS_SYSROOT}/usr/lib/ && \
#        ( cd ${CROSS_SYSROOT}/usr/lib/ && \
#          ln -s $f.1804.0 $f && \
#          chown crosstool:crosstool $f.1804.0 && \
#          chown -h crosstool:crosstool $f && \
#          chmod 0444 $f.1804.0 ) \
#    done && \
#    cp -r gstreamer-1.0 ${CROSS_SYSROOT}/usr/lib/ && \
#    chown -R crosstool:crosstool ${CROSS_SYSROOT}/usr/lib/gstreamer-1.0 && \
#    chmod -R 0444 ${CROSS_SYSROOT}/usr/lib/gstreamer-1.0 && \
#    cp -r gstreamer1.0 ${CROSS_SYSROOT}/usr/lib/ && \
#    chown -R crosstool:crosstool ${CROSS_SYSROOT}/usr/lib/gstreamer1.0 && \
#    chmod -R 0444 ${CROSS_SYSROOT}/usr/lib/gstreamer1.0 
    
# Building OpenCV for static use without FFMpeg.
# https://github.com/twistedfall/opencv-rust/issues/364#issuecomment-1308794985
# https://docs.opencv.org/4.x/d0/d76/tutorial_arm_crosscompile_with_cmake.html
# https://github.com/IntelRealSense/librealsense/issues/9962#issuecomment-998392844
# https://galaktyk.medium.com/how-to-build-opencv-with-gstreamer-b11668fa09c
# Flags: https://forums.raspberrypi.com/viewtopic.php?t=288404#p1743294
# https://forums.raspberrypi.com/viewtopic.php?t=290804#p1758436
# https://github.com/cross-rs/cross/blob/main/docker/Dockerfile.arm-unknown-linux-gnueabihf
# https://www.raspbian.org/RaspbianFAQ#What_compilation_options_should_be_set_Raspbian_code.3F

ARG CV=4.8.0
RUN cd /opt && \
   curl -L https://github.com/opencv/opencv/archive/${CV}.zip > opencv.zip && \
   curl -L https://github.com/opencv/opencv_contrib/archive/${CV}.zip > opencv_contrib.zip && \
   unzip opencv.zip && \
   unzip opencv_contrib.zip

RUN cd /opt && \
   mkdir opencv_build && \
   cd opencv_build && \
   cmake -DCMAKE_CXX_COMPILER=arm-unknown-linux-gnueabihf-g++ \
       -DCMAKE_C_COMPILER=arm-unknown-linux-gnueabihf-gcc \
       -DCMAKE_TOOLCHAIN_FILE=../opencv-${CV}/platforms/linux/arm-gnueabi.toolchain.cmake \
       -DCMAKE_C_FLAGS="${CROSS_CMAKE_OBJECT_FLAGS}" \
       -DCMAKE_CXX_FLAGS="${CROSS_CMAKE_OBJECT_FLAGS}" \
       -DCMAKE_BUILD_TYPE=Release \
       -DBUILD_SHARED_LIBS=NO \
       -DBUILD_DOCS=OFF \
       -DBUILD_EXAMPLES=OFF \
       -DBUILD_TESTS=OFF \
       -DBUILD_PERF_TESTS=OFF \
       -DBUILD_opencv_java=OFF \
       -DBUILD_opencv_python=OFF \
       -DWITH_GTK=OFF \
       -DWITH_FFMPEG=OFF \
       -DWITH_GSTREAMER=ON \
       -DCMAKE_INSTALL_PREFIX=/opt/opencv \
       -DOPENCV_EXTRA_MODULES_PATH=../opencv_contrib-${CV}/modules \
       ../opencv-${CV} && \
   sed -i.bak -e '1s/$/ -latomic/' apps/interactive-calibration/CMakeFiles/opencv_interactive-calibration.dir/link.txt && \
   make -j$(nproc)

# Install OpenCV and clean up
RUN cd /opt/opencv_build && \
   make install && \
   cd .. && \
   rm -r opencv.zip opencv_contrib.zip opencv-${CV} opencv_contrib-${CV} opencv_build /rpi-root

# Bug in opencv-rust builder, hunts for liblibopenjp2.a (and others)
# to build linker command, but then strips off both "lib"s and tries
# to link with -lopenjp2.  Workaround - link to the other version of the
# library name, so both can be found.
RUN bash -c 'for i in /opt/opencv/lib/opencv4/3rdparty/liblib*.a; do ln $i ${i/liblib/lib}; done'

# Tools required to build opencv-rust
RUN apt-get install -y clang libclang-dev lld git build-essential pkg-config

# Add OpenCV libraries to Rust build target
ENV CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_RUSTFLAGS="-Clink-arg=-L/opt/opencv/lib -Clink-arg=-L/opt/opencv/lib/opencv4/3rdparty $CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_RUSTFLAGS"
