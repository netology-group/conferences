FROM netologygroup/mqtt-gateway:v0.9.0 as mqtt-gateway-plugin
FROM netologygroup/janus-gateway:b4c77a8 as janus-conference-plugin
FROM debian:stretch

ENV DEBIAN_FRONTEND noninteractive

## -----------------------------------------------------------------------------
## Installing dependencies
## -----------------------------------------------------------------------------
RUN set -xe \
    && apt-get update \
    && apt-get -y --no-install-recommends install \
        apt-transport-https \
        ca-certificates \
        curl \
        less \
        libconfig-dev \
        libmicrohttpd-dev \
        libjansson-dev \
        libnice-dev \
        libcurl4-openssl-dev \
        libsofia-sip-ua-dev \
        libopus-dev \
        libogg-dev \
        libwebsockets-dev \
        libsrtp2-dev \
        gengetopt \
        libtool \
        automake \
        cmake \
        make \
        git \
        vim-nox \
    && PAHO_MQTT_BUILD_DIR=$(mktemp -d) \
        && PAHO_MQTT_VERSION='1.1.0' \
        && cd "${PAHO_MQTT_BUILD_DIR}" \
        && git clone "https://github.com/eclipse/paho.mqtt.c.git" . \
        && git checkout "v${PAHO_MQTT_VERSION}" \
        && make \
        && make install

## -----------------------------------------------------------------------------
## Installing Janus Gateway
## -----------------------------------------------------------------------------
ARG JANUS_GATEWAY_COMMIT='955069ae9441258bbc678b66bb58c7b326b1abd8'

RUN set -xe \
    && JANUS_GATEWAY_BUILD_DIR=$(mktemp -d) \
    && cd "${JANUS_GATEWAY_BUILD_DIR}" \
    && git clone 'https://github.com/netology-group/janus-gateway' . \
    && git checkout "${JANUS_GATEWAY_COMMIT}" \
    && ./autogen.sh \
    && ./configure --prefix='/opt/janus' \
    && make -j $(nproc) \
    && make install \
    && make configs \
    && rm -rf "${JANUS_GATEWAY_BUILD_DIR}"

COPY --from=janus-conference-plugin /opt/janus/lib/janus/plugins/*.so /opt/janus/lib/janus/plugins/

## -----------------------------------------------------------------------------
## Configuring Janus Gateway
## -----------------------------------------------------------------------------
COPY ./docker/configs/janus.jcfg /opt/janus/etc/janus/
COPY ./docker/configs/janus.transport.mqtt.jcfg /opt/janus/etc/janus/
COPY ./docker/configs/janus.plugin.conference.toml /opt/janus/etc/janus/

## -----------------------------------------------------------------------------
## Installing VerneMQ
## -----------------------------------------------------------------------------
RUN set -xe \
    && VERNEMQ_URI='https://github.com/vernemq/vernemq/releases/download/1.7.1/vernemq-1.7.1.stretch.x86_64.deb' \
    && VERNEMQ_SHA='f705246a3390c506013921e67b2701f28b9acbd6585a318cfc537a84ed430024' \
    && curl -fSL -o vernemq.deb "${VERNEMQ_URI}" \
        && echo "${VERNEMQ_SHA} vernemq.deb" | sha1sum -c - \
        && set +e; dpkg -i vernemq.deb || apt-get -y -f --no-install-recommends install; set -e \
    && rm vernemq.deb

COPY --from=mqtt-gateway-plugin "/app" "/app"

## -----------------------------------------------------------------------------
## Configuring VerneMQ
## -----------------------------------------------------------------------------
ENV APP_AUTHN_ENABLED "0"
ENV APP_AUTHZ_ENABLED "0"
RUN set -xe \
    && VERNEMQ_ENV='/usr/lib/vernemq/lib/env.sh' \
    && perl -pi -e 's/(RUNNER_USER=).*/${1}root\n/s' "${VERNEMQ_ENV}" \
    && VERNEMQ_CONF='/etc/vernemq/vernemq.conf' \
    && perl -pi -e 's/(listener.tcp.default = ).*/${1}0.0.0.0:1883\nlistener.ws.default = 0.0.0.0:8080/g' "${VERNEMQ_CONF}" \
    && perl -pi -e 's/(plugins.vmq_passwd = ).*/${1}off/s' "${VERNEMQ_CONF}" \
    && perl -pi -e 's/(plugins.vmq_acl = ).*/${1}off/s' "${VERNEMQ_CONF}" \
    && printf "\nplugins.mqttgw = on\nplugins.mqttgw.path = /app\n" >> "${VERNEMQ_CONF}"

## -----------------------------------------------------------------------------
## Install GStreamer
## -----------------------------------------------------------------------------
RUN set -xe \
    && apt-get install -y \
        gdb libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
        gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
        gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly \
        gstreamer1.0-libav libgstrtspserver-1.0-dev
