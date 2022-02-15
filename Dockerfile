FROM rust:latest as rust-env
WORKDIR /app
COPY server/. /app
RUN cargo build --release

# TODO: ADD NPM DEPENDENCIES

FROM debian:buster-slim
RUN apt-get update && apt-get install libssl-dev
# RUN npm install --global yarn
# RUN npm install --global yarn
COPY --from=rust-env /app/target/release/server /
CMD ["/server"]