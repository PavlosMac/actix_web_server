use actix_web::{dev::ResourceDef, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use futures::{stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
    domain: String,
}

pub async fn process_request(
    req: web::Json<Index>,
    data: web::Data<Mutex<AppState>>,
) -> Result<HttpResponse, AppError> {
    let domain = Url::parse(&req.domain).map_err(|e| AppError::BadRequest(e.to_string()))?;
    let cloned = domain.as_str().to_owned();

    let init = reqwest::get(domain.as_str())
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .text()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    actix_rt::spawn(async move {
        let d = domain.as_str();

        let indexables = process_domain_links(init.as_str(), d).await;

        if let Ok(mut state) = data.lock() {
            state.status.insert(d.to_string(), ProcessStatus::Pending);
            let results = stream::iter(indexables)
                .map(|url| async move {
                    match reqwest::get(&url).await {
                        Ok(r) => (url, r.status().to_string()),
                        Err(e) => (url, format!("Http error{}", e.to_string())),
                    }
                })
                .buffer_unordered(50)
                .collect::<Vec<(String, String)>>()
                .await;
            println!("finished... {}", &d);
            state.results.insert(d.to_string(), results);
            state.status.insert(d.to_string(), ProcessStatus::Complete);
        }
    });

    return Ok(HttpResponse::Accepted().body(format!("Processing.... {}", cloned)));
}

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
            return Ok(HttpResponse::Ok().json(scraping_results.clone()));
        } else {
            return Err(AppError::InternalServerError(format!(
                "Unable to retrieve key for {}",
                &req.domain
            )));
        }
    }
    return Ok(HttpResponse::Ok().body(format!("Status for {} - {:?}", &req.domain, &s)));
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
            .route("/results", web::get().to(get_results))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
