FROM rust:latest
WORKDIR /app/rpsBackend
RUN cargo install diesel_cli --no-default-features --features postgres
RUN echo "fn main() {}" > dummy.rs
COPY Cargo.toml .
RUN sed -i 's#src/bin/server.rs#dummy.rs#' Cargo.toml
RUN cargo build --release
RUN sed -i 's#dummy.rs#src/bin/server.rs#' Cargo.toml
COPY . .
RUN cargo run --bin generate_rsa_key_pair
EXPOSE 8080
RUN chmod +x ./entrypoint.sh
CMD [ "target/release/server" ]
