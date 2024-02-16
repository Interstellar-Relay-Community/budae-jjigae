FROM rust:1.76-alpine3.18 AS builder

RUN apk add --no-cache openssl libc-dev openssl-dev

RUN mkdir -p /opt/budae-jjigae
WORKDIR /opt/budae-jjigae

ADD . /opt/budae-jjigae

# Rust statically links to libc, but alpine does not like it
# (especially in multistage build)
# See https://github.com/sfackler/rust-native-tls/issues/190
ENV RUSTFLAGS=-Ctarget-feature=-crt-static -Ctarget-feature=+avx,+avx2,+fma
RUN cargo build --release

FROM alpine:3.18

RUN apk add --no-cache libgcc openssl ca-certificates tini

ENTRYPOINT ["/sbin/tini", "--"]

COPY --from=builder /opt/budae-jjigae/target/release/budae-jjigae /usr/bin/budae-jjigae

CMD ["/usr/bin/budae-jjigae"]
