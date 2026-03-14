FROM alpine:3.20

ARG TARGETPLATFORM

RUN apk add --no-cache ca-certificates tzdata unzip

WORKDIR /opt/ikuai-bypass

COPY docker/bin /opt/ikuai-bypass/docker/bin
COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
COPY config.yml /opt/ikuai-bypass/config.yml
COPY docker/frontends/app/dist /opt/ikuai-bypass/frontends/app/dist

RUN mkdir -p /etc/ikuai-bypass \
    && case "${TARGETPLATFORM}" in \
      "linux/amd64") src="linux-amd64" ;; \
      "linux/386") src="linux-386" ;; \
      "linux/arm64") src="linux-arm64" ;; \
      "linux/arm/v7") src="linux-arm7" ;; \
      "linux/arm/v6") src="linux-arm6" ;; \
      "linux/ppc64le") src="linux-ppc64le" ;; \
      "linux/riscv64") src="linux-riscv64" ;; \
      *) echo "unsupported TARGETPLATFORM=${TARGETPLATFORM}" >&2; exit 1 ;; \
    esac \
    && cp "/opt/ikuai-bypass/docker/bin/${src}/ikuai-bypass" /usr/local/bin/ikuai-bypass \
    && chmod +x /usr/local/bin/ikuai-bypass /usr/local/bin/docker-entrypoint.sh

VOLUME ["/etc/ikuai-bypass"]

EXPOSE 19001

ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["-r", "cron"]
