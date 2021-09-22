FROM alpine:edge AS build
MAINTAINER Ben <bencrulis@gmail.com>

RUN apk update && \
    apk add alpine-sdk linux-headers libpthread-stubs git build-base gcc musl musl-dev make libx11 libx11-dev glib glib-dev cairo cairo-dev \
    pango-dev fontconfig-dev libxinerama-dev libxfixes-dev libxcursor-dev \
    rust cargo &&\
    mkdir /visual_alarm_gui

COPY . /visual_alarm_gui

WORKDIR /visual_alarm_gui
ENV PKG_CONFIG_ALLOW_CROSS=1
CMD cargo build --target x86_64-alpine-linux-musl --release