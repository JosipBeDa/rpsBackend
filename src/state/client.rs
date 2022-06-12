use dotenv::dotenv;
use reqwest::Client;

pub fn initialize() -> Client {
    dotenv().ok();
    
    //Set the headers for the reqwest client builder, will need to be overriden if they mismatch
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "Content-type",
        "application/x-www-form-urlencoded".parse().unwrap(),
    );
    let client_builder = reqwest::ClientBuilder::new().default_headers(headers);
    client_builder.build().expect("Failed to build client")
}