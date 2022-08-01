FROM rust:latest
WORKDIR /rpsbackend
COPY . .
RUN cargo build --release
EXPOSE 8080
CMD cargo run --bin server
