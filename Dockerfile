FROM rust:latest as rust-env
WORKDIR /app
COPY server/. /app
RUN cargo build --release

# Install things required for SSL
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && apt-get install -y libssl-dev
RUN update-ca-certificates

# Install node CLI dependencies
RUN apt-get install -y curl
RUN curl -sL https://deb.nodesource.com/setup_16.x -o nodesource_setup.sh
RUN bash nodesource_setup.sh
RUN apt-get install -y nodejs
RUN npm i -g ipfs-car
RUN npm i -g carbites-cli
# Cleanup and run
RUN rm -rf /var/lib/apt/lists/*

CMD ["/app/target/release/server"]