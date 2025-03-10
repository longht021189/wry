use http::{Response, StatusCode};
use crate::RequestAsyncResponder;

pub static FILTERS: [&str; 18] = [
    "https://*.doubleclick.net/*",
    "https://*.fastclick.net/*",
    "https://www.googletagmanager.com/*",
    "https://cdn.ampproject.org/*",
    "https://*.scorecardresearch.com/*",
    "https://*.2mdn.net/*",
    "https://*.doubleverify.com/*",
    "https://*.ignimgs.com/*",
    "https://*.amazon-adsystem.com/*",
    "https://*.confiant-integrations.net/*",
    "https://*.jsdelivr.net/*",
    "https://*.ziffstatic.com/*",
    "https://*.adnxs.com/*",
    "https://*.chartbeat.net/*",
    "https://static.chartbeat.com/*",
    "https://zdbb.net/*",
    "https://*.zdbb.net/*",
    "https://*.liadm.com/*",
];

pub fn is_match(uri: &String) -> bool {
    (uri.contains("doubleclick.net/"))
    || (uri.contains("fastclick.net/"))
    || (uri.contains("cdn.ampproject.org"))
    || (uri.contains("www.googletagmanager.com"))
    || (uri.contains("scorecardresearch.com/"))
    || (uri.contains("2mdn.net/"))
    || (uri.contains("doubleverify.com/"))
    || (uri.contains("ignimgs.com/"))
    || (uri.contains("amazon-adsystem.com/"))
    || (uri.contains("confiant-integrations.net/"))
    || (uri.contains("jsdelivr.net/"))
    || (uri.contains("ziffstatic.com/"))
    || (uri.contains("adnxs.com/"))
    || (uri.contains("chartbeat.net/"))
    || (uri.contains("zdbb.net/"))
    || (uri.contains("liadm.com/"))
}

pub fn override_network_404(responder: RequestAsyncResponder) {
    let http_response = Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Vec::new())
        .expect("Failed to build 404 for ads");

    responder.respond(http_response);
}