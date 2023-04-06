FROM --platform=linux/amd64 ubuntu:16.04
ARG BUILD_ARCH
ARG RUST_TRIPLET

RUN sh -c 'dpkg --add-architecture $BUILD_ARCH'
RUN echo "deb [arch=amd64] http://uk.archive.ubuntu.com/ubuntu/ xenial main restricted universe multiverse" >> /etc/apt/sources.list
RUN echo "deb [arch=amd64] http://uk.archive.ubuntu.com/ubuntu/ xenial-updates main restricted universe multiverse" >> /etc/apt/sources.list
RUN sh -c 'echo "deb [arch=$BUILD_ARCH] http://ports.ubuntu.com/ubuntu-ports xenial main restricted universe multiverse"' >> /etc/apt/sources.list
RUN sh -c 'echo "deb [arch=$BUILD_ARCH] http://ports.ubuntu.com/ubuntu-ports xenial-updates main restricted universe multiverse"' >> /etc/apt/sources.list

RUN apt-get update -y || true
RUN apt search gcc

RUN apt-get install -y curl:amd64 gcc:amd64 pkg-config:amd64
RUN sh -c 'apt-get install -y libx11-dev:$BUILD_ARCH libasound2-dev:$BUILD_ARCH libudev-dev:$BUILD_ARCH'

RUN curl -sSf https://sh.rustup.rs | sh -s -- --default-toolchain nightly --profile minimal -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN printf "\
[build] \n\
rustflags = [\"-C\", \"embed-bitcode=off\"] \n\
[profile.dev] \n\
lto = \"off\" \
" > /root/.cargo/config

RUN sh -c 'rustup target add $RUST_TRIPLET'
RUN cargo install cargo-zigbuild

RUN curl -L https://ziglang.org/builds/zig-linux-x86_64-0.11.0-dev.2398+66520c834.tar.xz -o zig.tar.xz
RUN tar -xf zig.tar.xz
RUN rm zig.tar.xz
RUN mv zig-* /root/zig
ENV PATH="/root/zig:${PATH}"

WORKDIR /cix
COPY . .

RUN sh -c 'cargo zigbuild --target $RUST_TRIPLET'
