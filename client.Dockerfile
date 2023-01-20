FROM rust:1.66.1

RUN apt-get update && apt-get install -y protobuf-compiler

WORKDIR /usr/src/client
COPY . .

RUN cargo clean
RUN cargo install --path .

CMD ["client"]