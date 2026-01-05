FROM alpine:latest
 
WORKDIR /app
RUN wget --no-check-certificate -c -t3 -T60 -O ikuai-bypass.tar.gz https://github.com/dscao/ikuai-bypass/releases/download/v4.0.0/ikuai-bypass-${os}-${arch}.zip && \
  tar -zxvf ikuai-bypass.tar.gz && \
  rm -f ikuai-bypass.tar.gz

RUN apk add --no-cache tzdata
ENV TZ=Asia/Shanghai

CMD ["./ikuai-bypass", "-c", "/etc/ikuai-bypass/config.yml"]
