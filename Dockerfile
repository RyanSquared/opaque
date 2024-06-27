FROM scratch AS base

ARG PROFILE=dev

FROM base AS fetch
COPY --from=stagex/rust . /
COPY --from=stagex/busybox . /
COPY --from=stagex/musl . /
COPY --from=stagex/gcc . /
COPY --from=stagex/llvm . /
COPY --from=stagex/libunwind . /
COPY --from=stagex/openssl . /
COPY --from=stagex/zlib . /
COPY --from=stagex/ca-certificates . /
COPY . /opaque
WORKDIR opaque
RUN cargo fetch

FROM fetch AS build
COPY --from=stagex/binutils . /
ENV RUSTFLAGS='-C codegen-units=1 -C target-feature=+crt-static'
RUN --network=none \
    cargo build \
        --frozen \
        --profile $PROFILE \
        --target x86_64-unknown-linux-musl \
        --bin opaque

FROM build AS install
RUN <<-EOF
    set -eux
    mkdir -p /rootfs/usr/bin /rootfs/usr/share/opaque
    find /opaque/target -type f -executable -name opaque -exec cp {} /rootfs/usr/bin \;
EOF
COPY static /rootfs/usr/share/opaque/static
COPY content /rootfs/usr/share/opaque/content
COPY output_snippets /rootfs/usr/share/opaque/output_snippets


FROM stagex/filesystem AS package
EXPOSE 8000
COPY --from=install /rootfs/. /
# NOTE: Needed for `cp` in deployment
COPY --from=stagex/busybox . /
COPY --from=stagex/musl . /
WORKDIR /usr/share/opaque
ENTRYPOINT ["/usr/bin/opaque"]
