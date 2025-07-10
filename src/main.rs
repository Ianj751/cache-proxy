use std::sync::Mutex;

use actix_web::{
    web::{self, get, Data},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use awc::{body::BoxBody, Client};
use clap::{arg, Parser};

async fn send_req(req: HttpRequest, origin: &str) -> Result<HttpResponse, actix_web::Error> {
    let client = Client::default();
    let mut resp = client
        .request_from(origin, req.head())
        .send()
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;

    let mut client_resp = HttpResponse::new(resp.status());
    for (name, value) in resp.headers().iter() {
        client_resp
            .headers_mut()
            .append(name.clone(), value.clone());
    }
    let body = resp.body().await?;
    let client_resp = client_resp.set_body(body).map_into_boxed_body();

    Ok(client_resp)
}

async fn base_route(req: HttpRequest, data: web::Data<Mutex<MyData>>) -> impl Responder {
    let d = data.lock().unwrap();
    send_req(req, &d.origin).await
}
#[derive(Parser, Debug)]
struct CLI {
    #[arg(short, long, default_value = "8080")]
    port: u16,

    #[arg(short, long)]
    origin: String,
}
struct MyData {
    origin: String,
}
async fn start_server(ip: &str, port: u16, origin: &str) -> Result<(), std::io::Error> {
    let data = web::Data::new(Mutex::new(MyData {
        origin: origin.to_string(),
    }));
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .default_service(web::to(base_route))
    })
    .bind((ip, port))?
    .run()
    .await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cli = CLI::parse();
    start_server("127.0.0.1", cli.port, &cli.origin).await
}
