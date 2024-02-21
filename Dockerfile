FROM --platform=$TARGETPLATFORM rust:1.76-alpine3.18 AS builder

ARG TARGETPLATFORM
ARG TARGETARCH

RUN apk add --no-cache openssl libc-dev openssl-dev bash

# Prepare for UPX
ADD https://github.com/upx/upx/releases/download/v4.2.2/upx-4.2.2-"$TARGETARCH"_linux.tar.xz /opt/upx.tar.xz
RUN mkdir -p /opt/upx && tar xvf /opt/upx.tar.xz -C /opt/upx --strip-components=1

RUN mkdir -p /opt/budae-jjigae
WORKDIR /opt/budae-jjigae

ADD . /opt/budae-jjigae

RUN /opt/budae-jjigae/build.sh "$TARGETPLATFORM"

RUN strip /opt/budae-jjigae/target/release/budae-jjigae
RUN /opt/upx/upx /opt/budae-jjigae/target/release/budae-jjigae

FROM --platform=$TARGETPLATFORM alpine:3.18

RUN apk add --no-cache libgcc openssl ca-certificates tini

ENTRYPOINT ["/sbin/tini", "--"]

COPY --from=builder /opt/budae-jjigae/target/release/budae-jjigae /usr/bin/budae-jjigae

CMD ["/usr/bin/budae-jjigae"]
