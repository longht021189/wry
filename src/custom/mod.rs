use crate::{RequestAsyncResponder, Result};
use http::Request;

#[cfg(target_os = "windows")]
use webview2_com::Microsoft::Web::WebView2::Win32::{ICoreWebView2, COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL};

#[cfg(target_os = "windows")]
use windows_core::HSTRING;

mod ads;
mod mapgenie;
mod client;

pub fn is_custom_uri(uri: &String) -> bool {
    mapgenie::is_match(uri) ||
    ads::is_match(uri)
}

pub unsafe fn override_network(request: Request<Vec<u8>>, responder: RequestAsyncResponder) {
    let url = request.uri().to_string();

    if mapgenie::is_match(&url) {
        mapgenie::override_network(&url, request, responder);
    }
    else {
        ads::override_network_404(responder);
    }
}

#[cfg(target_os = "windows")]
pub unsafe fn setup_custom_protocol_handler(webview: &ICoreWebView2) -> Result<()> {
    let filters: Vec<String> = ads::FILTERS.iter()
        .chain(mapgenie::FILTERS.iter())
        .map(|s| s.to_string())
        .collect();

    for filter in filters {
        let filter = HSTRING::from(filter);
        webview.AddWebResourceRequestedFilter(&filter, COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL)?;
    }

    Ok(())
}
