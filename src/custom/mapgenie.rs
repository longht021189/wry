use std::fmt::Debug;
use http::{HeaderMap, Method, Request, Response, StatusCode};
use crate::RequestAsyncResponder;
use super::{ads, client};
use std::os::raw::c_char;
use std::ffi::{CStr, CString};
use std::str;

extern "C" {
    fn mapgenie_add_note(game_id: u64, map_id: u64, body: *const c_char, count: i32) -> *mut c_char;
    fn mapgenie_update_note(game_id: u64, map_id: u64, body: *const c_char, count: i32, note_id: *const c_char, note_id_len: i32) -> *mut c_char;
    fn mapgenie_delete_note(game_id: u64, map_id: u64, note_id: *const c_char, note_id_len: i32);

    fn mapgenie_add_location(game_id: u64, map_id: u64, location_id: u64);
    fn mapgenie_remove_location(game_id: u64, map_id: u64, location_id: u64);

    fn mapgenie_get_map_data(game_id: u64, map_id: u64, length: *mut usize) -> *mut c_char;
    fn free_buffer(ptr: *mut c_char);
}

macro_rules! option_to_result {
    ($option:expr, $error:expr) => {
        match $option {
            Some(value) => value,
            None => return Err(Box::new($error)),
        }
    };
}

pub static FILTERS: [&str; 3] = [
    "https://mapgenie.io/*/maps/*",
    "https://mapgenie.io/api/*",
    "https://mapgenie.io/inject/*",
];

static INJECT_DATA: [(&str, &str, &str); 2] = [
    ("mapgenie_script.js", "application/javascript", include_str!("mapgenie_script.js")),
    ("mapgenie_script.css", "text/css", include_str!("mapgenie_script.css")),
];

pub fn is_match(uri: &String) -> bool {
    uri.starts_with("https://mapgenie.io/")
}

pub unsafe fn override_network(url: &String, request: Request<Vec<u8>>, responder: RequestAsyncResponder) {
    if url.starts_with("https://mapgenie.io/inject/") {
        let name = url.replace("https://mapgenie.io/inject/", "");
        inject_data(name, responder);
    }
    else if url.starts_with("https://mapgenie.io/api/") {
        mapgenie_api(&url, request, responder);
    }
    else {
        mapgenie_html_maps(request, responder);
    }
}

fn inject_data(name: String, responder: RequestAsyncResponder) {
    for (key, content_type, value) in INJECT_DATA.iter() {
        if name == key.to_string() {
            let http_response = Response::builder()
                .header("content-type", content_type.to_string())
                .status(StatusCode::OK)
                .body(value.as_bytes().to_vec())
                .unwrap();

            responder.respond(http_response);
            return;
        }
    }

    ads::override_network_404(responder);
}

unsafe fn mapgenie_api(url: &String, request: Request<Vec<u8>>, responder: RequestAsyncResponder) {
    if url.starts_with("https://mapgenie.io/api/v1/user/locations/") {
        let location_id: u64 = url.split("/").last().unwrap().parse().unwrap();
        mapgenie_api_locations(location_id, request, responder);
    } else if url.starts_with("https://mapgenie.io/api/v1/user/notes") {
        mapgenie_api_notes(request, responder);
    } else if url.starts_with("https://mapgenie.io/api/local/map-data/") {
        mapgenie_api_locations_local(url, responder);
    }
}

unsafe fn mapgenie_api_locations_local(url: &String, responder: RequestAsyncResponder) {
    let parts: Vec<u64> = url
        .replace("https://mapgenie.io/api/local/map-data/", "")
        .split("/")
        .map(|x| x.parse::<u64>().unwrap())
        .collect();

    let mut length: usize = 0;
    let buffer = mapgenie_get_map_data(parts[0], parts[1], &mut length);

    if buffer.is_null() {
        ads::override_network_404(responder);
        return;
    }

    let result = CStr::from_ptr(buffer).to_str().unwrap();

    free_buffer(buffer);

    let http_response = Response::builder()
        .header("content-type", "application/json; charset=utf-8")
        .status(StatusCode::OK)
        .body(result.as_bytes().to_vec())
        .unwrap();

    responder.respond(http_response);
}

unsafe fn mapgenie_api_locations(id: u64, request: Request<Vec<u8>>, responder: RequestAsyncResponder) {
    let map_id: u64 = request.headers()
        .get("X-Map-ID")
        .expect("X-Map-ID not found")
        .to_str()
        .unwrap()
        .parse()
        .unwrap();

    let game_id: u64 = request.headers()
        .get("X-Game-ID")
        .expect("X-Game-ID not found")
        .to_str()
        .unwrap()
        .parse()
        .unwrap();

    match request.method().clone() {
        Method::PUT => { mapgenie_add_location(game_id, map_id, id); }
        Method::DELETE => { mapgenie_remove_location(game_id, map_id, id); }
        _ => {
            let http_response = Response::builder()
                .header("content-type", "application/json; charset=utf-8")
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body("{}".as_bytes().to_vec())
                .unwrap();

            responder.respond(http_response);
            return;
        }
    }

    let http_response = Response::builder()
        .header("content-type", "application/json; charset=utf-8")
        .status(StatusCode::OK)
        .body("{}".as_bytes().to_vec())
        .unwrap();

    responder.respond(http_response);
}

unsafe fn mapgenie_html_maps(request: Request<Vec<u8>>, responder: RequestAsyncResponder) {
    let method = request.method().clone();
    if method != Method::GET {
        ads::override_network_404(responder);
        return;
    }

    match client::process(request) {
        Ok((status, headers, body)) => {
            match edit_html_maps(&body) {
                Ok(body) => {
                    mapgenie_html_maps_response(status, headers, body, responder);
                }
                Err(e) => {
                    println!(">>>>>>>>>> Error: {:?}", e);
                    ads::override_network_404(responder);
                }
            }
        }
        Err(e) => {
            println!(">>>>>>>>>> Error: {:?}", e);
            ads::override_network_404(responder);
        }
    }
}

fn mapgenie_html_maps_response(status: StatusCode, headers: HeaderMap, body: String, responder: RequestAsyncResponder) {
    let mut http_response = Response::builder()
        .status(status)
        .body(body.into_bytes())
        .unwrap();

    *http_response.headers_mut() = headers;
    responder.respond(http_response);
}

fn edit_html_maps(html: &String) -> Result<String, Box<dyn Debug>> {
    let error: Box<dyn Debug> = Box::new("Start Not Found".to_string());
    let start = option_to_result!(html.find("<script>window.mapUrl"), error);
    let error: Box<dyn Debug> = Box::new("End Not Found".to_string());
    let end = option_to_result!(html[start..].find("</script>"), error) + start;
    let script_str = &html[(start + 8)..end];
    let script = String::from(script_str).replace("window.", "w.");

    Ok(format!(
        "{}\n{}{}\n<script>{}{}{}{}",
        &html[..start],
        "<link rel=\"stylesheet\" href=\"https://mapgenie.io/inject/mapgenie_script.css\" />",
        "<script type=\"application/javascript\" src=\"https://mapgenie.io/inject/mapgenie_script.js\"></script>",
        "const w = {};",
        script,
        "injectInputs(w);",
        &html[end..],
    ))
}

unsafe fn mapgenie_api_notes(request: Request<Vec<u8>>, responder: RequestAsyncResponder) {
    let map_id: u64 = request.headers()
        .get("X-Map-ID")
        .expect("X-Map-ID not found")
        .to_str()
        .unwrap()
        .parse()
        .unwrap();

    let game_id: u64 = request.headers()
        .get("X-Game-ID")
        .expect("X-Game-ID not found")
        .to_str()
        .unwrap()
        .parse()
        .unwrap();

    match request.method().clone() {
        Method::POST => {
            let body = String::from_utf8(request.body().clone())
                .expect("Invalid UTF-8");

            let body_length = body.len();

            let c_string = CString::new(body)
                .expect("Invalid UTF-8");

            let buffer = mapgenie_add_note(
                game_id, 
                map_id, 
                c_string.as_ptr(), 
                body_length.try_into().unwrap());

            if !buffer.is_null() {
                let result = CStr::from_ptr(buffer).to_str().unwrap();

                free_buffer(buffer);

                let http_response = Response::builder()
                    .header("content-type", "application/json; charset=utf-8")
                    .status(StatusCode::OK)
                    .body(result.as_bytes().to_vec())
                    .unwrap();

                responder.respond(http_response);
            } else {
                ads::override_network_404(responder);
            }
        }
        Method::PUT => {
            let node_id = request.uri().path().split("/").last().unwrap();
            let node_id_len = node_id.len();

            let body = String::from_utf8(request.body().clone())
                .expect("Invalid UTF-8");

            let body_length = body.len();

            let c_string = CString::new(body)
                .expect("Invalid UTF-8");

            let node_id_c = CString::new(node_id)
                .expect("Invalid UTF-8");

            let buffer = mapgenie_update_note(
                game_id, 
                map_id, 
                c_string.as_ptr(), 
                body_length.try_into().unwrap(),
                node_id_c.as_ptr(),
                node_id_len.try_into().unwrap());

            if !buffer.is_null() {
                let result = CStr::from_ptr(buffer).to_str().unwrap();

                free_buffer(buffer);

                let http_response = Response::builder()
                    .header("content-type", "application/json; charset=utf-8")
                    .status(StatusCode::OK)
                    .body(result.as_bytes().to_vec())
                    .unwrap();

                responder.respond(http_response);
            } else {
                ads::override_network_404(responder);
            }
        }
        Method::DELETE => {
            let node_id = request.uri().path().split("/").last().unwrap();
            let node_id_len = node_id.len();
            let c_string = CString::new(node_id)
                .expect("Invalid UTF-8");

            mapgenie_delete_note(game_id, map_id, c_string.as_ptr(), node_id_len.try_into().unwrap());

            let http_response = Response::builder()
                .header("content-type", "application/json; charset=utf-8")
                .status(StatusCode::NO_CONTENT)
                .body("".as_bytes().to_vec())
                .unwrap();

            responder.respond(http_response);
        }
        _ => {
            let http_response = Response::builder()
                .header("content-type", "application/json; charset=utf-8")
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body("{}".as_bytes().to_vec())
                .unwrap();

            responder.respond(http_response);
            return;
        }
    }
}