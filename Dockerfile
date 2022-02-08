FROM rust:1.58.1 as build
ENV PKG_CONFIG_ALLOW_CROSS=1

WORKDIR /usr/src/server
COPY server/. .

RUN cargo install --path .

FROM gcr.io/distroless/cc-debian10
COPY --from=build /usr/local/cargo/bin/server /usr/local/bin/server
CMD ["/usr/local/bin/server"]