use js_sys::{Date, Number};
use js_sys::Math::cos;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
//use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{CanvasRenderingContext2d, Crypto, HtmlCanvasElement};

use crate::geom::Point;
use crate::logger::{log_debug, log_error, log_info};
use crate::scene::LoopHandler;
use crate::scene::renderer::Sprite;

mod logger;
mod geom;
mod scene;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    log_info("Kosygin start");
    let window = web_sys::window().unwrap();
    match create_loading_canvases() {
        Ok(canvases) => {
            match scene::init_with_canvases(&window, canvases, Box::from(LoadingLoopHandler {})) {
                Ok(_) => log_info("Success"),
                Err(e) => log_error(format!("Failed to run, {:?}", &e).as_str())
            }
        }
        Err(e) => log_error(format!("Failed to create canvases {:?}", &e).as_str())
    }
    Ok(())
}

fn create_loading_canvases() -> Result<Vec<HtmlCanvasElement>, JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.create_element("canvas")?;
    let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>().unwrap();
    let sprite_width = 270_f32;
    let sprite_height = 80_f32;
    canvas.set_width(sprite_width as u32);
    canvas.set_height(sprite_height as u32);
    let context2d = canvas.get_context("2d")?.unwrap();
    let context2d = context2d.dyn_into::<CanvasRenderingContext2d>().unwrap();
    context2d.set_fill_style(&JsValue::from("lightgray"));
    context2d.fill_rect(0_f64, 0_f64, sprite_width as f64, sprite_height as f64);
    context2d.set_font("6em monospace");
    context2d.set_fill_style(&JsValue::from("black"));
    context2d.set_text_align("center");
    context2d.fill_text("Loading…", sprite_width as f64 * 0.5, 54_f64);
    log_info("Loading sprite ready");
    let mut canvases: Vec<HtmlCanvasElement> = Vec::with_capacity(1);
    canvases.push(canvas);
    Ok(canvases)
}

struct LoadingLoopHandler {}

impl LoopHandler for LoadingLoopHandler {
    fn handle_render_loop(&self, width: u32, height: u32) -> Box<dyn LoopHandler> {
        Box::from(LoadingLoopHandler {})
    }

    fn create_sprites(&self, width: u32, height: u32) -> Vec<Sprite> {
        log_debug("");
        let time = Date::now() as f64;
        let sprite_width = 270_f32;
        let sprite_height = 80_f32;
        let delta_x = 0.15_f32 * cos(time * 0.0065) as f32;
        let delta_y = delta_x * sprite_height / sprite_width;
        let sprite = Sprite {
            texture: 0,
            position: Point { x: width as f32 * 0.5 - delta_x, y: height as f32 * 0.33 - delta_y },
            pivot: Point { x: sprite_width * 0.5, y: sprite_height * 0.5 },
            rotation: 0_f32,
            width: sprite_width + delta_x * 2_f32,
            height: sprite_height + delta_y * 2_f32,
            alpha: 0.85 + 0.15 * cos(time * 0.03) as f32,
        };
        let mut sprites: Vec<Sprite> = Vec::with_capacity(1);
        sprites.push(sprite);
        sprites
    }
}

fn rand_range(crypto: &Crypto, min: f32, max: f32) -> Result<f32, JsValue> {
    let mut array = [0_u8; 4];
    crypto.get_random_values_with_u8_array(&mut array)?;
    let v = array[0] as f32 + (array[1] as f32 * 256_f32) + (array[2] as f32 * 65536_f32);
    let v = v / 16777216_f32;
    Ok(min + (max - min) * v)
}