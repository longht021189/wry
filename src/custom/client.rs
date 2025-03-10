use std::{fmt::Debug, sync::Arc};
use http::{HeaderMap, Request, StatusCode};
use reqwest::Url;

static mut CLIENT: Option<Arc<reqwest::blocking::Client>> = None;

unsafe fn get_client() -> Arc<reqwest::blocking::Client> {
    if let Some(client) = &CLIENT {
        return client.clone();
    }

    let client = Arc::new(reqwest::blocking::Client::new());
    CLIENT = Some(client.clone());
    client
}

fn map_error(e: url::ParseError) -> Box<dyn Debug> {
    Box::new(e)
}

fn map_error_2(e: reqwest::Error) -> Box<dyn Debug> {
    Box::new(e)
}

pub unsafe fn process(http_request: Request<Vec<u8>>) -> Result<(StatusCode, HeaderMap, String), Box<dyn Debug>> {
    let uri = http_request.uri().to_string();
    let url = Url::parse(&uri).map_err(map_error)?;
    let method = http_request.method().clone();
    let mut builder = get_client().request(method, url);

    builder = builder
        .headers(http_request.headers().clone());

    let reqwest_request = builder.build().map_err(map_error_2)?;
    let reqwest_response = get_client().execute(reqwest_request).map_err(map_error_2)?;
    let status = reqwest_response.status();
    let headers = reqwest_response.headers().clone();
    let body = reqwest_response.text().map_err(map_error_2)?;

    Ok((status, headers, body))
}