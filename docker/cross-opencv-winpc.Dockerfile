FROM ghcr.io/cross-rs/x86_64-pc-windows-gnu:0.2.5

# useful in case anything wants an interactive prompt
ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get install --assume-yes --no-install-recommends \
        xz-utils \
        cmake \
        unzip \
        python3 && \
    apt-get update

# Building OpenCV for static use without FFMpeg.
ARG CV=4.8.0
RUN cd /opt && \
   curl -L https://github.com/opencv/opencv/archive/${CV}.zip > opencv.zip && \
   curl -L https://github.com/opencv/opencv_contrib/archive/${CV}.zip > opencv_contrib.zip && \
   unzip opencv.zip && \
   unzip opencv_contrib.zip

# https://stackoverflow.com/questions/24193881/not-sure-how-to-build-opencv-for-mingw
RUN cd /opt && \
   mkdir opencv_build && \
   cd opencv_build && \
   cmake -DCMAKE_CXX_COMPILER="${CROSS_TOOLCHAIN_PREFIX}g++${CROSS_TOOLCHAIN_SUFFIX}" \
       -DCMAKE_C_COMPILER="${CROSS_TOOLCHAIN_PREFIX}gcc${CROSS_TOOLCHAIN_SUFFIX}" \
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
       ../opencv-${CV} 

RUN cd /opt/opencv_build && \
   make -j$(nproc)

# Install OpenCV and clean up
RUN cd /opt/opencv_build && \
   make install && \
   cd .. && \
   rm -r opencv.zip opencv_contrib.zip opencv-${CV} opencv_contrib-${CV} opencv_build

# Bug in opencv-rust builder, hunts for liblibopenjp2.a (and others)
# to build linker command, but then strips off both "lib"s and tries
# to link with -lopenjp2.  Workaround - link to the other version of the
# library name, so both can be found.
RUN bash -c 'for i in /opt/opencv/lib/opencv4/3rdparty/liblib*.a; do ln $i ${i/liblib/lib}; done'

# Tools required to build opencv-rust
RUN apt-get install -y clang libclang-dev lld git build-essential pkg-config

# Add OpenCV libraries to Rust build target
ENV CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS="-Clink-arg=-L/opt/opencv/lib -Clink-arg=-L/opt/opencv/lib/opencv4/3rdparty $CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS"
