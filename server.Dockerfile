FROM rust:1.66.1

RUN apt-get update && apt-get install -y protobuf-compiler netcat

WORKDIR /usr/src/server
COPY . .

RUN cargo clean
RUN cargo install --path .

EXPOSE 50051

CMD ["server"]