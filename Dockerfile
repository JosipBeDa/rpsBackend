FROM rust:latest
WORKDIR /app/rpsBackend
RUN cargo install diesel_cli --no-default-features --features postgres
COPY . .
RUN diesel migration run
RUN cargo build --release
RUN cargo run --bin generate_rsa_key_pair
EXPOSE 8080
CMD cargo run --bin server
