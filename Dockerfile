FROM alpine:latest

ARG VERSION
ARG TARGETOS
ARG TARGETARCH

WORKDIR /app
RUN apk add --no-cache tzdata unzip wget

RUN echo "Downloading version: ${VERSION} for ${TARGETOS}/${TARGETARCH}" && \
    wget --no-check-certificate -c -t3 -T60 -O ikuai-bypass.zip \
    "https://github.com/dscao/ikuai-bypass/releases/download/${VERSION}/ikuai-bypass-${TARGETOS}-${TARGETARCH}.zip" && \
    unzip ikuai-bypass.zip && \
    rm -f ikuai-bypass.zip

ENV TZ=Asia/Shanghai

CMD ["./ikuai-bypass", "-c", "/etc/ikuai-bypass/config.yml"]
