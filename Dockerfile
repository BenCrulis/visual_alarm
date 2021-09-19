FROM alpine:edge AS build
MAINTAINER Ben <bencrulis@gmail.com>

RUN apk update && \
    apk add alpine-sdk linux-headers libpthread-stubs git build-base gcc musl musl-dev make libx11 libx11-dev glib glib-dev cairo cairo-dev \
    rust cargo &&\
    mkdir /visual_alarm

COPY . /visual_alarm

WORKDIR /visual_alarm
ENV RUSTFLAGS="-C target-feature=+crt-static -L/lib64 -l pthread"
CMD cargo build --target x86_64-alpine-linux-musl --release