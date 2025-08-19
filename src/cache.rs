use std::str::FromStr;

use actix_http::{
    header::{HeaderName, HeaderValue},
    StatusCode,
};
use actix_web::{
    web::{self},
    HttpRequest, HttpResponse, HttpResponseBuilder,
};
use anyhow::Ok;
use awc::body::{BoxBody, MessageBody};
use base64::{engine::general_purpose::STANDARD, Engine};
use redis::{Commands, FromRedisValue};

use sha2::{Digest, Sha256};

pub fn start_cache() -> anyhow::Result<redis::Connection> {
    let uri_scheme = "redis";
    let redis_host_name = "localhost:6379";

    let redis_conn_url = format!("{}://{}", uri_scheme, redis_host_name);

    Ok(redis::Client::open(redis_conn_url)
        .expect("Invalid connection URL")
        .get_connection()?)
}

/**
 * Item is present -> Ok(Some(http_response))
 * Item is absent -> Ok(None)
 */
pub fn check_cache(
    request: HttpRequest,
    body: web::Bytes,
    conn: &mut redis::Connection,
) -> anyhow::Result<Option<HttpResponse<BoxBody>>> {
    let cache_key = http_req_to_string(request, body)?;
    let value = conn.get(cache_key).expect("sdfasfsdf");
    let val = match value {
        redis::Value::Nil => Ok(String::new()),
        _ => Ok(FromRedisValue::from_redis_value(&value).expect("boiasfyhiofa")),
    }?;

    if val.is_empty() {
        return anyhow::Ok(None);
    }

    let val = string_to_http_resp(val).expect("hi2");

    anyhow::Ok(Some(val))
}

pub fn set_cache_val(
    key: HttpRequest,
    key_body: web::Bytes,
    val: HttpResponse<BoxBody>,
    conn: &mut redis::Connection,
) -> anyhow::Result<String> {
    let serialized_request = http_req_to_string(key, key_body)?;
    let serialized_response = http_resp_to_string(val)?;

    let ttl_s: u64 = 300;
    let () = conn.set_ex(&serialized_request, serialized_response, ttl_s)?;

    anyhow::Ok(serialized_request)
}
// "{method}|{http_version}|{path w/ query string}|{headers}|{sha256 body}"
pub fn http_req_to_string(request: HttpRequest, body: web::Bytes) -> anyhow::Result<String> {
    let res = Sha256::digest(body);
    let hash_result = format!("{:x}", res);

    let head = request.head();
    let method = head.method.clone();
    let uri = head.uri.path_and_query();
    let version = head.version;
    let headers = head.headers.clone();

    let h: Vec<(String, String)> = headers
        .iter()
        .map(|(name, value)| {
            (
                name.as_str().to_string(),
                value.to_str().unwrap_or("invalid header value").to_string(),
            )
        })
        .collect();
    let headers_string = serde_json::to_string(&h)?;

    match uri {
        Some(path) => anyhow::Ok(format!(
            "METHOD:{}|VERSION:{:?}|PATH:{}|HEADERS:{}|BODY_SHA256:{}",
            method, version, path, headers_string, hash_result
        )),
        None => anyhow::Ok(format!(
            "METHOD:{}|VERSION:{:?}|HEADERS:{}|BODY_SHA256:{}",
            method, version, headers_string, hash_result
        )),
    }
}
//"STATUS:{}|HEADERS:{}|BODY_BASE64:{}"
fn http_resp_to_string(response: HttpResponse) -> anyhow::Result<String> {
    let status = response.status().as_u16();
    let headers = response.headers().clone();

    let h: Vec<(String, String)> = headers
        .iter()
        .map(|(name, value)| {
            (
                name.as_str().to_string(),
                value.to_str().unwrap_or_default().to_string(),
            )
        })
        .collect();
    let headers_string = serde_json::to_string(&h)?;

    let body = response.into_body().try_into_bytes().unwrap();
    let base64_body = STANDARD.encode(body);

    anyhow::Ok(format!(
        "STATUS:{}|HEADERS:{}|BODY_BASE64:{}",
        status, headers_string, base64_body
    ))
}

fn string_to_http_resp(serialized_response: String) -> anyhow::Result<HttpResponse<BoxBody>> {
    let resp: Vec<&str> = serialized_response.split('|').collect();
    if resp.len() != 3 {
        return Err(anyhow::anyhow!("invalid request serialized request size"));
    }

    let status_code = resp[0];
    let headers = resp[1];
    let base64_body = resp[2];

    if !status_code.starts_with("STATUS:") {
        return Err(anyhow::anyhow!(
            "invalid request format: does not contain 'STATUS' prefix"
        ));
    } else if !headers.starts_with("HEADERS:") {
        return Err(anyhow::anyhow!(
            "invalid request format: does not contain 'HEADERS' prefix"
        ));
    } else if !base64_body.starts_with("BODY_BASE64:") {
        return Err(anyhow::anyhow!(
            "invalid request format: does not contain 'BODY_BASE64' prefix"
        ));
    }

    //there's no way that find should return none based on the above lines
    // so unwrap is not really a problem here
    let status_index = status_code.find(':').unwrap();
    let s = status_code.get(status_index + 1..).unwrap();
    let status_code = StatusCode::from_u16(s.parse::<u16>().unwrap()).unwrap();

    let body_index = base64_body.find(":").unwrap();
    let base64_body = base64_body.get(body_index + 1..).unwrap();

    // AHHHHH
    let header_index = headers.find(":").unwrap();
    let headers = headers.get(header_index + 1..).unwrap();
    let h: Vec<(String, String)> = serde_json::from_str(headers)?;

    let mut builder = HttpResponseBuilder::new(status_code);
    for (name, val) in h {
        let header_name = HeaderName::from_str(&name).unwrap();
        let header_value = HeaderValue::from_str(&val).unwrap();
        builder.append_header((header_name, header_value));
    }

    let body_bytes = STANDARD.decode(base64_body).unwrap_or_default();
    let resp = builder.body(body_bytes);
    anyhow::Ok(resp.map_into_boxed_body())
}
