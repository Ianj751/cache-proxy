use std::sync::Mutex;

use actix_web::{
    middleware::Logger,
    web::{self},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};

use anyhow::Ok;
use awc::Client;
use clap::{arg, Parser};
use env_logger::Env;

use crate::cache::{check_cache, set_cache_val};

mod cache;

async fn send_req(
    req: HttpRequest,
    origin: &str,
    conn: &Mutex<redis::Connection>,
    request_body: web::Bytes,
) -> actix_web::Result<HttpResponse> {
    let client: Client = Client::default();
    let mut connection = match conn.lock() {
        std::result::Result::Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let val = check_cache(req.clone(), request_body.clone(), &mut connection)
        .map_err(actix_web::error::ErrorInternalServerError)?;
    if let Some(resp) = val {
        return actix_web::Result::Ok(resp);
    }

    let mut resp = client
        .request_from(origin, req.head())
        .send_body(request_body.clone())
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;

    let mut client_resp = HttpResponse::new(resp.status());
    for (name, value) in resp.headers().iter() {
        client_resp
            .headers_mut()
            .append(name.clone(), value.clone());
    }
    let body = resp
        .body()
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;
    let client_resp: HttpResponse = client_resp.set_body(body).map_into_boxed_body();

    //this is so bad but i dont feel like dealing with the borrow checker rn
    let mut cached_resp = HttpResponse::new(client_resp.status());

    for (name, value) in client_resp.headers().iter() {
        cached_resp
            .headers_mut()
            .append(name.clone(), value.clone());
    }
    let _ = set_cache_val(req, request_body, cached_resp, &mut connection);

    actix_web::Result::Ok(client_resp)
}

async fn base_route(req: HttpRequest, data: web::Data<MyData>, body: web::Bytes) -> impl Responder {
    send_req(req, &data.origin, &data.conn, body).await
}
#[derive(Parser, Debug)]
struct Cli {
    /// Port to run proxy server on
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// URL to direct requests to
    #[arg(short, long)]
    origin: String,
}
struct MyData {
    origin: String,
    conn: Mutex<redis::Connection>,
}
async fn start_server(ip: &str, port: u16, origin: &str) -> anyhow::Result<()> {
    let connection = cache::start_cache()?;

    let data = web::Data::new(MyData {
        origin: origin.to_string(),
        conn: Mutex::new(connection),
    });

    env_logger::init_from_env(Env::default().default_filter_or("info"));
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("[%t] %a | %r status: %s | %D ms"))
            .app_data(data.clone())
            .default_service(web::to(base_route))
    })
    .bind((ip, port))?
    .run()
    .await?;

    Ok(())
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    start_server("127.0.0.1", cli.port, &cli.origin).await
}
