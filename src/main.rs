use actix_web::{web, App, HttpResponse, HttpServer};
use futures::{stream, StreamExt};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use url::Url;

mod domain;
use domain::*;

mod errors;
use errors::*;

#[derive(PartialEq, Debug)]
enum ProcessStatus {
    Pending,
    Error,
    Complete,
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
#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
    domain: String,
}
/// POST request route handler, parse as domain using regex, add protocol, parse as Url
/// request page doc, spawn new futures thread for parallel bulky IO, save state to global AppState in HashMap
pub async fn process_request(
    req: web::Json<Index>,
    data: web::Data<Mutex<AppState>>,
) -> Result<HttpResponse, AppError> {
    let with_proto = check_protocol(String::from(&req.domain));
    info!("With protocol {}", with_proto);
    let url = Url::parse(with_proto.as_str()).unwrap();

    let host = url.to_owned();

    actix_rt::spawn(async move {
        let host_base = match host.host_str() {
            Some(v) => v,
            _ => "",
        };
        let page_results = do_urls(with_proto.as_str(), host_base).await;
        info!("Indexing {} number of domain urls.", page_results.len());
        let cloned_key = req.domain.to_owned();
        if let Ok(mut state) = data.lock() {
            state.status.insert(cloned_key, ProcessStatus::Pending);

            let mut results = stream::iter(page_results)
                .map(|url| async move {
                    match reqwest::get(&url).await {
                        Ok(r) => {
                            info!("Request complete for url: {}", &url);
                            (url, r.status().to_string())
                        }
                        Err(e) => (url, format!("Http error: {}", e)),
                    }
                })
                .buffer_unordered(50)
                .collect::<Vec<(String, String)>>()
                .await;
            info!("Processing complete for {}", &req.domain);
            results.push((String::from("Indexed Count"), results.len().to_string()));
            state.results.insert(String::from(&req.domain), results);
            state
                .status
                .insert(String::from(&req.domain), ProcessStatus::Complete);
        }
    });
    Ok(HttpResponse::Accepted().body("Processing entity..."))
}
/// GET request route handler, open global lock, check for results of domain processing, return AppError or HttpResponse Ok with results as Vec<(T,T)>
async fn get_results(
    req: web::Query<Index>,
    data: web::Data<Mutex<AppState>>,
) -> Result<HttpResponse, AppError> {
    let data = data
        .lock()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let s = data
        .status
        .get(&req.domain)
        .unwrap_or(&ProcessStatus::Error);

    if matches!(*s, ProcessStatus::Complete) {
        let scraping_results = data.results.get(&req.domain);
        if scraping_results.is_some() {
            return Ok(HttpResponse::Ok().json(scraping_results));
        } else {
            return Err(AppError::InternalServerError(format!(
                "Unable to retrieve key for {}",
                &req.domain
            )));
        }
    }
    Ok(HttpResponse::Ok().body(format!("Status for {} - {:?}", &req.domain, &s)))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let data = web::Data::new(Mutex::new(AppState::default()));
    env_logger::init();
    info!("....Start Actix Web Server....");
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/process", web::post().to(process_request))
            .route("/results", web::get().to(get_results))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
