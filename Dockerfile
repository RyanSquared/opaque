FROM rust:1.64 AS builder

# Note: When in release mode, both of these should be `release`. When in dev
# mode, these should both be `debug`.
ARG PROFILE=dev
ARG TARGET=debug
ADD . /build
WORKDIR /build

RUN --mount=type=cache,target=/build/target cargo build --profile $PROFILE

FROM debian:bullseye
COPY --from=builder /build/target/ /target
EXPOSE 8000
