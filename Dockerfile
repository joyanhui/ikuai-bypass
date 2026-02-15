FROM --platform=$BUILDPLATFORM golang:1.25-alpine AS builder

ARG TARGETOS
ARG TARGETARCH
ARG TARGETVARIANT
ARG VERSION=dev

WORKDIR /src

COPY go.mod go.sum ./
RUN go mod download

COPY . .

RUN set -eux; \
    if [ "${TARGETARCH}" = "arm" ]; then export GOARM="${TARGETVARIANT#v}"; fi; \
    CGO_ENABLED=0 GOOS="${TARGETOS}" GOARCH="${TARGETARCH}" \
    go build -trimpath -ldflags="-s -w -X main.version=${VERSION}" -o /out/ikuai-bypass ./

FROM alpine:latest

WORKDIR /opt/ikuai-bypass

RUN apk add --no-cache tzdata
ENV TZ=Asia/Shanghai

COPY --from=builder /out/ikuai-bypass /usr/local/bin/ikuai-bypass
COPY config_example.yml /opt/ikuai-bypass/config.yml
COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

EXPOSE 19001

ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["ikuai-bypass", "-c", "/etc/ikuai-bypass/config.yml", "-r", "cron"]
