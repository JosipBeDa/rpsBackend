#!/bin/bash

# Generate RSA key pair
cargo run --bin generate_rsa_key_pair

# Run Migrations
diesel migration run

# Start the server
cargo run --bin server