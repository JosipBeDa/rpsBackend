use reqwest::Client;

pub fn initialize() -> Client {
    let client_builder = reqwest::ClientBuilder::new();
    client_builder.build().expect("Failed to build client")
}