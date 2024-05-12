pub mod proxy;
use std::io::Read;
use std::{fs::File, net::TcpListener};

use actix_web::{dev::Server, web, App, HttpResponse, HttpServer};
use pingora::prelude::*;
use scraper::{Html, Selector};

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

async fn crawl() -> HttpResponse {
    //let html = client::proxy::client().await;
    log::debug!("lets start");
    let x = parse_me();
    HttpResponse::Ok().finish()
}

pub fn parse_me() -> std::io::Result<()> {
    let mut file = File::open("src/x.html")?;
    let mut html = String::new();
    if let Ok(_size) = file.read_to_string(&mut html) {
        log::debug!("{}", html);
    };
    let document = Html::parse_document(&html);
    let livescore_selector = Selector::parse("div.livescores-comp status-fixture").unwrap();
    for i in document.select(&livescore_selector) {
        //let country_selector = Selector::parse("i.")
        log::debug!("{}", i.inner_html());
    }

    Ok(())
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/get_matches", web::get().to(crawl))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
