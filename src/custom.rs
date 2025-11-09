use http::{Request, Response as HttpResponse};
use once_cell::sync::Lazy;
use std::{
    borrow::Cow,
    sync::{Arc, RwLock},
};

type NavigationHandlerFn = dyn Fn(String) -> bool;
type OverrideNavigationHandlerFn = dyn Fn(String, &NavigationHandlerFn) -> bool + Send + Sync;
type RequestHandlerFn = dyn Fn(&str, Request<Vec<u8>>, bool) -> Option<HttpResponse<Cow<'static, [u8]>>>;
type OverrideRequestHandlerFn = dyn Fn(&str, Request<Vec<u8>>, bool, &RequestHandlerFn) -> Option<HttpResponse<Cow<'static, [u8]>>> + Send + Sync;

static NAVIGATION_HANDLER_OVERRIDE: Lazy<RwLock<Option<Arc<OverrideNavigationHandlerFn>>>> = Lazy::new(|| RwLock::new(None));

static REQUEST_HANDLER_OVERRIDE: Lazy<RwLock<Option<Arc<OverrideRequestHandlerFn>>>> = Lazy::new(|| RwLock::new(None));

pub fn get_navigation_handler(navigation_handler: Option<Box<dyn Fn(String) -> bool>>) -> Option<Box<NavigationHandlerFn>> {
    let base_handler: Box<NavigationHandlerFn> =
        navigation_handler.unwrap_or_else(|| Box::new(|_| true));

    Some(Box::new(move |url| {
        let override_handler = NAVIGATION_HANDLER_OVERRIDE
            .read()
            .expect("override navigation handler lock poisoned")
            .clone();

        if let Some(handler) = override_handler {
            handler(url, &*base_handler)
        } else {
            base_handler(url)
        }
    }))
}

pub fn get_request_handler(request_handler: Box<RequestHandlerFn>) -> Box<RequestHandlerFn> {
    Box::new(move |webview_id, request, is_document_start_script_enabled| {
        let override_handler = REQUEST_HANDLER_OVERRIDE
            .read()
            .expect("override request handler lock poisoned")
            .clone();

        if let Some(handler) = override_handler {
            handler(
                webview_id,
                request,
                is_document_start_script_enabled,
                &*request_handler,
            )
        } else {
            request_handler(webview_id, request, is_document_start_script_enabled)
        }
    })
}

pub fn override_navigation_handler<F: Fn(String, &NavigationHandlerFn) -> bool + Send + Sync + 'static>(handler: F) {
    let mut guard = NAVIGATION_HANDLER_OVERRIDE
        .write()
        .expect("override navigation handler lock poisoned");
    *guard = Some(Arc::new(handler));
}

pub fn override_request_handler<F: Fn(&str, Request<Vec<u8>>, bool, &RequestHandlerFn) -> Option<HttpResponse<Cow<'static, [u8]>>> + Send + Sync + 'static>(handler: F) {
    let mut guard = REQUEST_HANDLER_OVERRIDE
        .write()
        .expect("override request handler lock poisoned");
    *guard = Some(Arc::new(handler));
}

pub fn clear_navigation_handler_override() {
    let mut guard = NAVIGATION_HANDLER_OVERRIDE
        .write()
        .expect("override navigation handler lock poisoned");
    guard.take();
}

pub fn clear_request_handler_override() {
    let mut guard = REQUEST_HANDLER_OVERRIDE
        .write()
        .expect("override request handler lock poisoned");
    guard.take();
}
