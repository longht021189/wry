use webview2_com::Microsoft::Web::WebView2::Win32::{ICoreWebView2, COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL};
use windows_core::HSTRING;
use super::super::Result;

mod constants {
  pub static INSTANCE: super::Custom = super::Custom {};
  pub const URI: &str = "*";
}

pub use constants::INSTANCE;

pub struct Custom {
}

impl Custom {
  pub fn init_webview(&self, webview: &ICoreWebView2) -> Result<()> {
    unsafe {
      self.unsafe_init_webview(webview)
    }
  }

  unsafe fn unsafe_init_webview(&self, webview: &ICoreWebView2) -> Result<()> {
    // webview.AddWebResourceRequestedFilter(&HSTRING::from(URI), COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL)?;
    
    Ok(())
  }
}