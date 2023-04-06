FROM --platform=amd64 ubuntu:16.04
ARG BUILD_ARCH
ARG RUST_TRIPLET

RUN dpkg --add-architecture $BUILD_ARCH
RUN echo "deb [arch=$BUILD_ARCH] http://ports.ubuntu.com/ubuntu-ports xenial main restricted universe multiverse" \
    >> /etc/apt/sources.list
RUN echo "deb [arch=$BUILD_ARCH] http://ports.ubuntu.com/ubuntu-ports xenial-updates main restricted universe multiverse" \
    >> /etc/apt/sources.list

RUN apt-get update -qqy || true
RUN apt-get install -qqy \
    curl pkg-config gcc \
    libx11-dev:$BUILD_ARCH libasound2-dev:$BUILD_ARCH libudev-dev:$BUILD_ARCH

RUN curl -sSf https://sh.rustup.rs | sh -s -- --default-toolchain nightly --profile minimal -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN printf "\
[build] \n\
rustflags = [\"-C\", \"embed-bitcode=off\"] \n\
[profile.dev] \n\
lto = \"off\" \
" > /root/.cargo/config

RUN rustup target add $RUST_TRIPLET
RUN cargo install cargo-zigbuild

RUN curl -L https://ziglang.org/builds/zig-linux-x86_64-0.11.0-dev.2398+66520c834.tar.xz -o zig.tar.xz
RUN tar -xf zig.tar.xz
RUN rm zig.tar.xz
RUN mv zig-* /root/zig
ENV PATH="/root/zig:${PATH}"

WORKDIR /cix
COPY . .

RUN cargo zigbuild --target $RUST_TRIPLET
