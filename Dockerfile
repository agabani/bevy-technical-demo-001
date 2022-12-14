# 1: Build
FROM rust:1.66.0-alpine3.16 as builder

# 1a: Prepare toolchain
RUN apk update && \
    apk add alsa-lib-dev libudev-zero-dev musl-dev pkgconfig

# 1b: Download and compile Rust dependencies using fake source code and store as a separate Docker layer
WORKDIR /home/appuser/app

COPY .docker/main.rs src/
COPY .docker/lib.rs src/

COPY .docker/main.rs examples/quic_client.rs
COPY .docker/main.rs examples/quic_server.rs
COPY .docker/main.rs examples/tls.rs

COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml

RUN cargo build --release

# 1c: Build the application using the real source code
COPY . .

RUN cargo build --release --features "server"

# 2: Copy the excutable and extra files to an empty Docker image
FROM scratch

USER 10000:10000

COPY --chown=0:0 --from=builder /home/appuser/app/target/release/bevy-technical-demo /

CMD [ "/bevy-technical-demo" ]
