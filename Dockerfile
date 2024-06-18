FROM registry.hub.docker.com/library/rust:1.71 AS builder

# Note: When in release mode, both of these should be `release`. When in dev
# mode, PROFILE is dev, TARGET is debug
ARG PROFILE=dev
ARG TARGET=debug
ADD . /build
WORKDIR /build

RUN \
    --mount=type=cache,target=/build/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --profile $PROFILE --features cache && \
    mkdir -p /out && \
    find /build/target/$TARGET -type f -executable -name opaque -exec cp {} /out \;

FROM debian:bullseye
COPY --from=builder /out/opaque /usr/local/bin/opaque

WORKDIR /usr/share/opaque

# Load static content and posts
COPY static /usr/share/opaque/static
COPY content /usr/share/opaque/content
COPY output_snippets /usr/share/opaque/output_snippets

EXPOSE 8000
ENTRYPOINT ["/usr/local/bin/opaque"]
