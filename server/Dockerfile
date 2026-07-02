FROM crpi-jya41iij3cykq66t.cn-chengdu.personal.cr.aliyuncs.com/jiangker_love/debian:stable-20240904-slim-shanghai

RUN apt-get update && apt-get -y install ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY ./target/x86_64-unknown-linux-musl/release/push-hub-server .
RUN chmod +x ./push-hub-server

EXPOSE 3000

ENV SERVER_HOST=0.0.0.0 \
    SERVER_PORT=3000

CMD ["./push-hub-server"]
