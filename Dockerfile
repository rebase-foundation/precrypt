FROM rust:1.58.1

WORKDIR /usr/src/server
COPY . .

RUN cargo build --release

CMD ["./target/release/server"]