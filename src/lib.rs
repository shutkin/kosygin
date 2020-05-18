use std::cell::RefMut;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{console, Crypto, Response, Blob, ImageBitmap, HtmlCanvasElement, MouseEvent, TouchEvent, CanvasRenderingContext2d};
use js_sys::{Date, Number};

use logger::{log_debug, log_info};
use renderer::{Renderer, TextureAtlas, Projection, Sprite};
use resource_manager::ImageLoader;
use wasm_bindgen::__rt::std::rc::Rc;
use wasm_bindgen::__rt::core::cell::RefCell;
use crate::geom::Point;
use wasm_bindgen::__rt::core::f32::consts::PI;
use crate::logger::log_error;
use crate::Stage::{LOADING, SNOWFLAKES};
use js_sys::Math::cos;

mod logger;
mod scene;

const IMAGES_URL: [&str; 6] = ["/img/snowflake0.png", "/img/snowflake1.png", "/img/snowflake2.png",
    "/img/snowflake3.png", "/img/snowflake4.png", "/img/snowflake5.png"];

#[derive(Clone, Copy)]
enum Stage {
    LOADING, SNOWFLAKES
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    spawn_local(run_logged());
    Ok(())
}

async fn run_logged() {
    match run().await {
        Ok(_) => log_info("Success"),
        Err(e) => console::log_1(&e)
    }
}

async fn run() -> Result<(), (JsValue)> {
    let window = web_sys::window().unwrap();


    log_info("Start fetch images");
    for image in IMAGES_URL.iter() {
        log_info(format!("Start fetch image {}", image).as_str());
        let response = ImageLoader::fetch_image(image)?.await?;
        let response: Response = response.dyn_into().unwrap();
        log_info(format!("Image {} fetched", image).as_str());
        let blob = JsFuture::from(response.blob()?).await?;
        let blob: Blob = blob.dyn_into::<Blob>().unwrap();
        log_info(format!("Image {} blob read", image).as_str());
        let image_bitmap = JsFuture::from(window.create_image_bitmap_with_blob(&blob)?).await?;
        let image_bitmap: ImageBitmap = image_bitmap.dyn_into::<ImageBitmap>().unwrap();
        log_info(format!("Image {} decoded", image).as_str());
        context_rc.borrow_mut().images.push(image_bitmap);
    }

    create_scene(context_rc.borrow_mut());
    Ok(())
}

fn create_loading_scene(mut context: RefMut<SceneContext>) -> Result<(), JsValue> {
    log_info("Create loading scene");
    let full_width = context.renderer_context.projection.canvas_width;
    let full_height = context.renderer_context.projection.canvas_height;
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
    context.renderer_context.atlas = context.renderer_context.renderer.create_texture_with_canvases(&document, &canvases)?;
    context.sprites.clear();
    context.sprites.push(Sprite {
        texture: 0,
        position: Point { x: full_width as f32 * 0.5, y: full_height as f32 * 0.33 },
        pivot: Point { x: sprite_width * 0.5, y: sprite_height * 0.5 },
        rotation: 0_f32,
        width: sprite_width,
        height: sprite_height,
        alpha: 1_f32
    });
    log_info("Loading sprite added to scene");
    Ok(())
}

fn render_loop() {
    match context.stage {
        LOADING => render_loop_loading(context),
        SNOWFLAKES => render_loop_snowflakes(context)
    }
}

fn render_loop_loading(mut context: RefMut<SceneContext>) {
    let time = Date::now() as f64;
    match context.sprites.get_mut(0) {
        Some(sprite) => {
            let delta_x = 0.15_f32 * cos(time * 0.0065) as f32;
            let delta_y = delta_x * sprite.height / sprite.width;
            sprite.alpha = 0.85 + 0.15 * cos(time * 0.03) as f32;
            sprite.width += delta_x * 2_f32;
            sprite.position.x -= delta_x;
            sprite.height += delta_y * 2_f32;
            sprite.position.y -= delta_y;
        }
        None => {}
    }
    context.renderer_context.renderer.render(
        &context.renderer_context.projection, &context.sprites, &context.renderer_context.atlas);
}

fn render_loop_snowflakes(mut context: RefMut<SceneContext>) {
    let height = context.renderer_context.projection.canvas_height as f32;
    let width = context.renderer_context.projection.canvas_width as f32;

    let time = Date::now() as u64;
    let delta_seconds = if context.last_render_time > 0 {
        (time - context.last_render_time) as f32 / 1000_f32
    } else { 0_f32 };
    context.last_render_time = time;

    let mouse_delta = match context.mouse_pos {
        Some(p) => p - match context.last_render_mouse_pos {
            Some(l) => l,
            None => p
        },
        None => Point { x: 0.0, y: 0.0 }
    };
    let mouse_delta = Point { x: mouse_delta.x / width, y: mouse_delta.y / height };
    context.last_render_mouse_pos = context.mouse_pos.clone();
    let wind_factor = if delta_seconds > 1.0 { 0.0 } else { 1.0 - 0.5 * delta_seconds };
    context.wind = (context.wind + mouse_delta * 350.0 * delta_seconds) * wind_factor;

    let wind = context.wind.clone();
    for i in 0..context.sprites.len() {
        let mut sprite = &mut context.sprites[i];
        sprite.position.x += delta_seconds * (wind.x + 0.1) * sprite.width;
        if sprite.position.x > width + 100.0 {
            sprite.position.x -= width + 200.0;
        }
        if sprite.position.x < -100.0 {
            sprite.position.x += width + 200.0;
        }
        sprite.position.y += delta_seconds * (wind.y * 0.5 + 0.33) * sprite.width;
        if sprite.position.y > height + 100.0 {
            sprite.position.y -= height + 200.0;
        }
        if sprite.position.y < -100.0 {
            sprite.position.y += height + 200.0;
        }
        let rotation_speed = match i % 5 {
            0 => -0.05_f32,
            1 => -0.025_f32,
            2 => 0.0_f32,
            3 => 0.025_f32,
            _ => 0.05_f32
        };
        sprite.rotation += delta_seconds * PI * rotation_speed;
        if sprite.rotation > 2.0 * PI {
            sprite.rotation -= 2.0 * PI;
        }
    }
}

fn rand_range(crypto: &Crypto, min: f32, max: f32) -> Result<f32, JsValue> {
    let mut array = [0_u8; 4];
    crypto.get_random_values_with_u8_array(&mut array)?;
    let v = array[0] as f32 + (array[1] as f32 * 256_f32) + (array[2] as f32 * 65536_f32);
    let v = v / 16777216_f32;
    Ok(min + (max - min) * v)
}