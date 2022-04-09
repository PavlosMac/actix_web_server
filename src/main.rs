use actix_web::{dev::ResourceDef, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use futures::{stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use url::Url;
mod domain;
use domain::*;

#[derive(PartialEq, Debug)]
enum ProcessStatus {
    Error(String),
    Complete,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
    domain: String,
}

pub async fn process_request(
    req: web::Json<Index>,
    data: web::Data<Mutex<AppState>>,
) -> HttpResponse {
    println!("this stuff ==> {:?}", req.domain);

    actix_rt::spawn(async move {
        let mut data = data.lock().unwrap();

        let init = reqwest::get("www.rust-lang.org")
            .await
            .unwrap()
            .text()
            .await;

        println!("{}", init.unwrap());

        let origin = check_protocol("https://www.rust-lang.org".to_owned());
        let domain = Url::parse(&origin);
        let origin = domain.unwrap().host().unwrap().to_string();

        // let u = process_domain_links(init.unwrap().as_str(), origin).await;
        // let o = u.unwrap();
        let o = vec!["https://"];
        // println!("got links {:?}", o);
        // //////////
        // let p = stream::iter(o)
        //     .map(|url| async move {
        //         // println!("url ==> {}", url);
        //         match reqwest::get(&url).await {
        //             Ok(r) => (url, r.status().to_string()),
        //             Err(e) => (url, format!("Error {}", e.to_string())),
        //         }
        //     })
        //     .buffer_unordered(10)
        //     .collect::<Vec<(String, String)>>()
        //     .await;
        // data.results.insert(String::from("rust-lang.org"), p);
        // data.status
        //     .insert(String::from("rust-lang.org"), ProcessStatus::Complete);
    });

    return HttpResponse::Accepted().body("Processing....");
}

async fn shorter_request(data: web::Data<Mutex<AppState>>) -> HttpResponse {
    println!("shorter requests");
    // let d = data.lock();
    if let Ok(d) = data.lock() {
        if let Some(s) = d.status.get("rust-lang.org") {
            if *s == ProcessStatus::Complete {
                let results = d.results.get("rust-lang.org").unwrap().clone();
                return HttpResponse::Ok().json(results);
            }
        }
    };
    return HttpResponse::Ok().body("Processing....");
}

fn check_protocol(org: String) -> String {
    if !org.contains("https://") {
        let mut u = String::from("https://");
        u.push_str(&org.to_string());
        return u;
    }
    org
}

#[derive(Debug)]
pub struct AppState {
    results: HashMap<String, Vec<(String, String)>>,
    status: HashMap<String, ProcessStatus>,
}

impl AppState {
    fn default() -> Self {
        Self {
            results: HashMap::new(),
            status: HashMap::new(),
        }
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let data = web::Data::new(Mutex::new(AppState::default()));

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/process", web::post().to(process_request))
            .route("/shorter_request", web::get().to(shorter_request))

        // .app_data(data.clone())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
