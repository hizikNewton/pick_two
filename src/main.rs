use std::net::TcpListener;

use pick_two::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    env_logger::init();
    let listener = TcpListener::bind("127.0.0.1:8000").expect("Failed to bind random port");
    run(listener)?.await
}
