use wasm_bindgen_futures::JsFuture;
use web_sys::{RequestInit, RequestMode, Request};
use wasm_bindgen::JsValue;

pub struct SceneLoader {
    images: Vec<ImageBitmap>
}

fn fetch_image(src: &str) -> Result<JsFuture, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::SameOrigin);
    let request = Request::new_with_str_and_init(src, &opts)?;
    request.headers().set("Accept", "image/*")?;
    let window = web_sys::window().unwrap();
    Ok(JsFuture::from(window.fetch_with_request(&request)))
}
