pub mod parsers;
pub mod proxy;

use std::io::Read;
use std::{fs::File, net::TcpListener};

use actix_web::{dev::Server, web, App, HttpResponse, HttpServer};
use parsers::parse_details::parse_details;
use pingora::prelude::*;
use proxy::client;

use scraper::selectable::Selectable;
use scraper::{Html, Selector};

/* struct Match {
    team_a: String,
    team_b: String,
    match_link: String,
}

impl Match {
    pub fn new(team_a: String, team_b: String, match_link: String) -> Self {
        Match {
            team_a,
            team_b,
            match_link,
        }
    }
}
 */
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

async fn crawl() -> HttpResponse {
    //let html = client::proxy::client().await;

    /*  if let Ok(x) = parse_me() {
        x.matches
            .iter()
            .enumerate()
            .for_each(|(idx, link)| println!("{idx} {link}"))
    }
    crawl_details().await;*/
    let _c = parse_details::game_selector().unwrap();

    HttpResponse::Ok().finish()
}

async fn crawl_details() {
    log::debug!("lets start");
    let match_detail = client::proxy::client(
        "/matches/2024/05/12/england/premier-league/manchester-united-fc/arsenal-fc/4082380/"
            .to_string(),
    )
    .await;
    println!("{:?}", match_detail.unwrap())
}

#[derive(Debug)]
pub struct MatchGroup {
    matches: Vec<String>,
    closed_match_link: Vec<String>,
}

impl MatchGroup {
    pub fn new() -> Self {
        MatchGroup {
            matches: Vec::new(),
            closed_match_link: Vec::new(),
        }
    }
}

pub fn parse_me() -> std::io::Result<MatchGroup> {
    let mut file = File::open("src/x.html")?;
    let mut html = String::new();

    if let Ok(_size) = file.read_to_string(&mut html) {
        log::debug!("succesfully read to string");
    };
    let mut match_group = MatchGroup::new();
    let document = Html::parse_document(&html);

    let livescore_selector = Selector::parse("div.livescores-comp").unwrap();

    let h2_selector = Selector::parse("h2").unwrap();

    let data_comp_selector = Selector::parse("div.comp_content ol").unwrap();

    let a_selector = Selector::parse("a").unwrap();

    for livescore in document.select(&livescore_selector) {
        let livescore_frag = Html::parse_fragment(&livescore.inner_html());

        if let Some(matches) = livescore_frag.select(&data_comp_selector).next() {
            let li_selector = Selector::parse("li.status-fixture").unwrap();

            for matchh in matches.select(&li_selector) {
                let teams = Selector::parse("div.teams").unwrap();
                if let Some(team) = matchh.select(&teams).next() {
                    let team_frag = Html::parse_fragment(&team.inner_html());
                    let match_link = team_frag
                        .select(&a_selector)
                        .next()
                        .unwrap()
                        .value()
                        .attr("href");
                    if let Some(link) = match_link {
                        match_group.matches.push(link.to_string());
                    }
                }
            }
        } else {
            for closed_links_area_comp in livescore_frag.select(&h2_selector) {
                let comp_name = closed_links_area_comp
                    .select(&a_selector)
                    .next()
                    .unwrap()
                    .attr("href");
                if let Some(link) = comp_name {
                    match_group.closed_match_link.push(link.to_string());
                }
            }
        }
    }

    Ok(match_group)
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
