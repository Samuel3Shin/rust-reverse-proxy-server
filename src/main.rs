use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::sync::RwLock;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Result};
use dotenv::dotenv;

#[macro_use(lazy_static)]
extern crate lazy_static;

lazy_static! {
    static ref CACHE: RwLock<HashMap<String, CacheItem>> = {
        HashMap::new().into()
    };
}

struct CacheItem {
    result: String,
    timestamp: Instant,
}

async fn insert_cache(url: &str, cached_data: CacheItem) {
    CACHE.write().await.insert(url.to_owned(), cached_data);
}

async fn check_cache(url:&str) -> String{
    let cache = CACHE.read().await;
    if let Some(cached_item) = cache.get(url) {
        return cached_item.result.clone();
    }
    "".to_owned()
}

async fn update_cache_timestamp(url:&str) {
    let mut cache = CACHE.write().await;
    cache.get_mut(url).unwrap().timestamp = Instant::now();
}

async fn remove_old_cache(cache_lifetime:Duration) {
    let mut cache = CACHE.write().await;
    cache.retain(|_, val| val.timestamp.elapsed()<cache_lifetime);
}

async fn handle_request(
    req: HttpRequest,
    client: web::Data<reqwest::Client>,
) -> Result<HttpResponse, Error> {
    let uri = req.uri();
    let url_with_slash = format!("{}", uri);
    let url = &url_with_slash[1..];

    let cached_result = check_cache(url).await;
    if !cached_result.is_empty() {
        println!("Cache hit!");
        update_cache_timestamp(url).await;
        return Ok(HttpResponse::Ok().body(cached_result));
    }

    match client.get(url).send().await {
        Ok(response) => {
            match response.text().await {
                Ok(body) => {
                    let cached_data = CacheItem{result:body.clone(), timestamp:Instant::now()};
                    insert_cache(url, cached_data).await;
                    Ok(HttpResponse::Ok().body(body))
                }
                Err(error) => Err(actix_web::error::ErrorBadRequest(error))
            }
        }
        Err(error) => Err(actix_web::error::ErrorBadRequest(error))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let local_host_ip = std::env::var("LOCAL_HOST_IP").expect("LOCAL_HOST_IP must be set.");
    let local_host_port = std::env::var("LOCAL_HOST_PORT").expect("LOCAL_HOST_PORT must be set.")
                                .parse::<u16>().unwrap();
    let remove_old_cache_interval = std::env::var("REMOVE_OLD_CACHE_INTERVAL")
                                        .expect("REMOVE_OLD_CACHE_INTERVAL must be set.").parse::<u64>().unwrap();
    let cache_lifetime = std::env::var("CACHE_LIFETIME").expect("CACHE_LIFETIME must be set.").parse::<u64>().unwrap();
    
    let one_second = Duration::new(remove_old_cache_interval, 0);
    let mut remove_old_cache_interval = tokio::time::interval(one_second);
    let thirty_seconds = Duration::new(cache_lifetime, 0);
    
    tokio::task::spawn(async move {
        loop {
            remove_old_cache_interval.tick().await;
            remove_old_cache(thirty_seconds).await;
        }
    });

    let reqwest_client = reqwest::Client::default();

    HttpServer::new(move || 
        {
        App::new()
        .app_data(web::Data::new(reqwest_client.clone()))
        .default_service(web::to(handle_request))
        })
        .bind((local_host_ip, local_host_port))?
        .run()
        .await
}