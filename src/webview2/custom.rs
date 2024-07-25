use webview2_com::Microsoft::Web::WebView2::Win32::{ICoreWebView2, ICoreWebView2Environment, ICoreWebView2WebResourceRequestedEventArgs, COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL};
use windows::Win32::UI::Shell::SHCreateMemStream;
use windows_core::HSTRING;
use crate::error;
use std::{fs, process::Command};
use super::super::Result;

mod constants {
  pub static INSTANCE: super::Custom = super::Custom {};
  pub const URI: &str = "*";
  pub static mut LOCATIONS: Vec<String> = Vec::new();
  pub static mut LOADED: bool = false;
}

pub use constants::INSTANCE;

pub struct Custom {
}

impl Custom {
  pub fn init_webview(&self, webview: &ICoreWebView2) -> Result<()> {
    println!("init_webview");

    unsafe {
      self.unsafe_init_webview(webview)
    }
  }

  pub fn custom_filter(&self, source: HSTRING) -> HSTRING {
    println!("custom_filter - source: {:?}", source);
    HSTRING::from(constants::URI)
  }

  unsafe fn unsafe_init_webview(&self, webview: &ICoreWebView2) -> Result<()> {
    // webview.AddWebResourceRequestedFilter(&HSTRING::from(constants::URI), COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL)?;
    // Ok(())

    todo!()
  }

  unsafe fn load(&self) {
    if constants::LOADED {
      return;
    }

    let data = fs::read_to_string("C:/Users/longh/OneDrive/Desktop/projects/github.com/longht021189/workspace/scripts/locations.txt").unwrap_or("".to_string());
    if !data.is_empty() {
      constants::LOCATIONS = data.split(',').map(|id| id.to_string()).collect();
    }
    constants::LOADED = true;
  }

  unsafe fn save(&self) {
    fs::write("C:/Users/longh/OneDrive/Desktop/projects/github.com/longht021189/workspace/scripts/locations.txt", constants::LOCATIONS.join(",")).expect("")
  }

  pub unsafe fn handle_request(
    &self, 
    uri: &str, 
    args: &ICoreWebView2WebResourceRequestedEventArgs, 
    env: &ICoreWebView2Environment
  ) -> Result<()> {
    let hostname = uri.split('/').nth(2).unwrap();
    if hostname.ends_with("chartbeat.net") || 
        hostname.ends_with("doubleclick.net") || 
        hostname.ends_with("google-analytics.com") || 
        hostname.ends_with("googletagmanager.com") || 
        hostname.ends_with("scorecardresearch.com") || 
        hostname.ends_with("ziffstatic.com") || 
        hostname.ends_with("zdbb.net") {
      return Err(error::Error::InitScriptError);
    }
    
    if uri.starts_with("https://mapgenie.io/api/v1/user/locations/") {
      let id = uri.strip_prefix("https://mapgenie.io/api/v1/user/locations/").unwrap();

      self.load();
      if constants::LOCATIONS.contains(&id.to_string()) {
        constants::LOCATIONS.retain(|x| x != id);
      } else {
        constants::LOCATIONS.push(id.to_string());
      }
      self.save();

      let status = http::status::StatusCode::OK;
      let status_code = status.as_u16();
      let status = HSTRING::from(status.canonical_reason().unwrap_or("OK"));
      let stream = SHCreateMemStream(Some(b"{}"));
      let response = env.CreateWebResourceResponse(stream.as_ref(), status_code as i32, &status, None)?;
      args.SetResponse(&response)?;
      return Ok(());
    }
    
    if uri == "https://mapgenie.io/elden-ring/maps/the-lands-between" {
      self.load();

      let cmd = Command::new("node")
        .arg("C:/Users/longh/OneDrive/Desktop/projects/github.com/longht021189/workspace/scripts/a.js")
        .output()?;

      let data = String::from_utf8(cmd.stdout).expect("");
      let mut data = data.split("=======");
      let headers_map = HSTRING::from(data.next().unwrap());
      let html = data.next().unwrap();
      let locations = (&constants::LOCATIONS).iter().map(|id| format!("\"{}\":true", id));
      let locations = locations.collect::<Vec<String>>().join(",");
      let locations = format!("\"locations\":{{{}}}", locations);
      let game_locations_count = format!("\"gameLocationsCount\":{}", constants::LOCATIONS.len());
      let all = vec![
        "\"id\":6557285",
        "\"role\":\"user\"",
        &locations,
        &game_locations_count,
        "\"hasPro\":true",
        "\"trackedCategoryIds\":[]",
        "\"suggestions\":[]",
        "\"presets\":[]"
      ];
      let all = format!("window.user = {{{}}};", all.join(","));
      let html = html.replace("window.user = null;", &all);
      let html = html.replace("</head>", "<style>#blobby-left { display: none; }</style></head>");

      let status = http::status::StatusCode::OK;
      let status_code = status.as_u16();
      let status = HSTRING::from(status.canonical_reason().unwrap_or("OK"));
      let stream = SHCreateMemStream(Some(html.as_bytes()));
      let response = env.CreateWebResourceResponse(stream.as_ref(), status_code as i32, &status, &headers_map)?;
      args.SetResponse(&response)?;
      return Ok(());
    }
    
    // https://mapgenie.io/api/v1/user/categories
    // "body": "{\"map_id\":413,\"category\":5699}", "method": "POST"

    Ok(())
  }
}