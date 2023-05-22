// Dependencies
#[macro_use] extern crate rocket;
mod logreader;

use clap::Parser;
use base64::{Engine as _, engine::general_purpose};
use logreader::LogReader;
use rocket::{http::{Status, ContentType}, serde::Serialize};

// Main endpoint
#[derive(Serialize)]
struct LogResponse {
    success: bool,
    length: usize,
    data: Option<String>,
    next_key: Option<String>
}

#[get("/log/<path>/<retrieval_key>")]
fn log(path: String, retrieval_key: String) -> (Status, (ContentType, String)) {
    // Attempt to decode the base64 path
    let path = match general_purpose::URL_SAFE_NO_PAD.decode(path) {
        Ok(path_buf) => match String::from_utf8(path_buf) {
            Ok(path) => path,
            Err(_) => return (Status::BadRequest, (ContentType::Text, String::from("invalid characters within path")))
        },
        Err(_) => return (Status::BadRequest, (ContentType::Text, String::from("unable to decode path")))
    };

    // Attempt to read the log file
    let mut reader = LogReader::default();
    let log_info = reader.read_file(path, retrieval_key);

    // Generate the response
    let content = log_info.content;
    let got_content = content.is_some();
    let response = LogResponse {
        success: got_content,
        length: if got_content { content.as_ref().unwrap().len() } else { 0 },
        data: content,
        next_key: log_info.next_key
    };

    // Return
    (Status::Ok, (ContentType::JSON, serde_json::to_string(&response).unwrap()))
}

// CLI args
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // The host of the IW4M server
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    host: String,

    // Specify a custom port to connect the server to
    #[arg(short, long, default_value = "1625")]
    port: u16
}

// Main function
#[rocket::main]
async fn main() {
    // Parse the arguments
    let args = Args::parse();

    // Run the API server
    let config = rocket::Config::figment()
        .merge(("port", args.port))
        .merge(("address", args.host));

    rocket::custom(config)
        .mount("/", routes![log])
        .launch()
        .await.unwrap();
}