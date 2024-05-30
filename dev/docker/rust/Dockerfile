################################################################################
# Rust image
################################################################################
FROM rust:1.78.0

RUN apt-get update

RUN rustup install nightly
RUN cargo install --locked cargo-watch cargo-udeps cargo-crev
RUN cargo install --locked cargo-audit --features=fix
