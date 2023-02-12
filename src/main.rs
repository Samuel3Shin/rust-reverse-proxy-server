use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::Mutex;

use actix_web::{
    web, App, Error, HttpRequest, HttpResponse,
    HttpServer, Result
};

#[macro_use(lazy_static)]
extern crate lazy_static;

struct CacheItem {
    result: String,
    timestamp: std::time::Instant,
}

lazy_static! {
    static ref CACHE: Mutex<HashMap<String, CacheItem>> = {
        let hm = HashMap::new();
        Mutex::new(hm)
    };
}

fn check_cache(url:&str) -> String{
    let cache = CACHE.lock().unwrap();
    if let Some(cached_item) = cache.get(url) {
        println!("Cache hit!");
        return cached_item.result.clone();
    }
    "".to_string()
}

fn insert_cache(url:&str, cached_data:CacheItem) {
    let mut cache = CACHE.lock().unwrap();
    cache.insert((*url).to_string(), cached_data);
}

async fn handle_request(
    req: HttpRequest,
    client: web::Data<reqwest::Client>,
) -> Result<HttpResponse, Error> {
    let uri = req.uri();
    let url_with_slash = format!("{}", uri);
    let url = &url_with_slash[1..];

    let cached_result = check_cache(url);
    if !cached_result.is_empty() {
        return Ok(HttpResponse::Ok().body(cached_result));
    }

    match client.get(url).send().await {
        Ok(response) => {
            match response.text().await {
                Ok(body) => {
                    let cached_data = CacheItem{result:body.clone(), timestamp:Instant::now()};
                    insert_cache(url, cached_data);
                    Ok(HttpResponse::Ok().body(body))
                }
                Err(error) => Err(actix_web::error::ErrorBadRequest(error))
            }
        }
        Err(error) => Err(actix_web::error::ErrorBadRequest(error))
    }
}

async fn remove_old_cache(thirty_seconds:Duration) {
    let mut cache = CACHE.lock().unwrap();
    // for (key, val) in cache.iter() {
    //     println!("{}", key);
    //     println!("{:?}", val.timestamp.elapsed());
    //     println!("{:?}", val.timestamp.elapsed()>Duration::new(10, 0));
    // }

    cache.retain(|_, val| val.timestamp.elapsed()<thirty_seconds);
    // println!("haha!");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let one_second = Duration::new(1, 0);
    let mut interval = tokio::time::interval(one_second);
    let thirty_seconds = Duration::new(10, 0);
    
    tokio::task::spawn(async move {
        loop {
            interval.tick().await;
            remove_old_cache(thirty_seconds).await;
        }
    });

    let reqwest_client = reqwest::Client::default();

    HttpServer::new(move|| 
        {
        App::new()
        .app_data(web::Data::new(reqwest_client.clone()))
        .default_service(web::to(handle_request))
        })
        .bind(("127.0.0.1", 8081))?
        .run()
        .await
}