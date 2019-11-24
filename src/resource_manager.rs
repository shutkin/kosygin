use wasm_bindgen::JsValue;
use web_sys::{RequestInit, RequestMode, Request};
use wasm_bindgen_futures::JsFuture;

pub struct ImageLoader {
}

impl ImageLoader {
    pub fn fetch_image(src: &str) -> Result<JsFuture, JsValue> {
        let mut opts = RequestInit::new();
        opts.method("GET");
        opts.mode(RequestMode::SameOrigin);
        let request = Request::new_with_str_and_init(src, &opts)?;
        request.headers().set("Accept", "image/png")?;
        let window = web_sys::window().unwrap();
        Ok(JsFuture::from(window.fetch_with_request(&request)))
    }
}
