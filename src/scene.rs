use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, MouseEvent, TouchEvent};
use js_sys::Number;

use crate::geom::Point;
use crate::logger::{log_debug, log_error, log_info};
use crate::scene::renderer::{Projection, Renderer, Sprite, TextureAtlas};

mod scene_loader;
pub mod renderer;

pub trait LoopHandler {
    fn handle_render_loop(&self, width: u32, height: u32) -> Box<dyn LoopHandler>;
    fn create_sprites(&self, width: u32, height: u32) -> Vec<Sprite>;
}

struct RendererContext {
    renderer: Renderer,
    atlas: TextureAtlas,
    projection: Projection,
}

struct SceneContext {
    renderer_context: RendererContext,
    mouse_pos: Option<Point>,
}

pub fn init_with_canvases(window: &web_sys::Window, canvases: Vec<HtmlCanvasElement>, loop_handler: Box<impl LoopHandler + 'static>) -> Result<(), JsValue> {
    let context = SceneContext {
        renderer_context: create_renderer_with_canvases(canvases.clone())?,
        mouse_pos: None,
    };
    let context_rc = Rc::new(RefCell::new(context));
    request_animation_frame(context_rc.clone(), loop_handler)?;

    // resize
    {
        let context_rc = context_rc.clone();
        let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
            match { create_renderer_with_canvases(canvases.clone()) } {
                Ok(renderer_context) => {
                    (*context_rc).borrow_mut().renderer_context = renderer_context;
                }
                Err(e) => log_error(format!("Failed to create renderer, {:?}", &e).as_str())
            };
        }) as Box<dyn Fn(_)>);
        window.add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref());
        closure.forget();
    }

    // touch events
    {
        let context_rc = context_rc.clone();
        let closure = Closure::wrap(Box::new(move |e: MouseEvent| {
            mouse_move_handler((*context_rc).borrow_mut(), e);
        }) as Box<dyn Fn(MouseEvent)>);
        window.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref());
        closure.forget();
    }
    {
        let context_rc = context_rc.clone();
        let closure = Closure::wrap(Box::new(move |e: TouchEvent| {
            touch_move_handler((*context_rc).borrow_mut(), e);
        }) as Box<dyn Fn(TouchEvent)>);
        window.add_event_listener_with_callback("touchstart", closure.as_ref().unchecked_ref());
        window.add_event_listener_with_callback("touchmove", closure.as_ref().unchecked_ref());
        window.add_event_listener_with_callback("touchend", closure.as_ref().unchecked_ref());
        closure.forget();
    }
    Ok(())
}

fn mouse_move_handler(mut context: RefMut<SceneContext>, e: MouseEvent) {
    context.mouse_pos = if e.buttons() == 1 {
        Some(Point { x: e.client_x() as f32, y: e.client_y() as f32 })
    } else { None };
    log_debug(format!("Mouse: {:?}", &context.mouse_pos).as_str());
}

fn touch_move_handler(mut context: RefMut<SceneContext>, e: TouchEvent) {
    context.mouse_pos = match e.touches().get(0) {
        Some(t) => Some(Point { x: t.client_x() as f32, y: t.client_y() as f32 }),
        None => None
    };
    log_debug(format!("Mouse: {:?}", &context.mouse_pos).as_str());
}

fn create_renderer_with_canvases(canvases: Vec<HtmlCanvasElement>) -> Result<RendererContext, JsValue> {
    let window = web_sys::window().unwrap();
    let pixel_ratio = window.device_pixel_ratio();
    let pixel_ratio = if pixel_ratio < 1.0 { 1.0 } else { pixel_ratio };
    let window_width = window.inner_width()?;
    let window_width: Number = window_width.dyn_into::<Number>()?;
    let width = (window_width.value_of() * pixel_ratio) as u32;
    let window_height = window.inner_height()?;
    let window_height: Number = window_height.dyn_into::<Number>()?;
    let height = (window_height.value_of() * pixel_ratio) as u32;

    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>()?;
    canvas.set_width(width);
    canvas.set_height(height);
    log_info(format!("Canvas sizes: {}x{}, pixel ratio {}", width, height, pixel_ratio).as_str());
    let renderer = Renderer::init(&canvas)?;
    let atlas = renderer.create_texture_with_canvases(&document, &canvases)?;
    let projection = Projection::create(width, height);
    Ok(RendererContext { renderer, atlas, projection })
}

fn request_animation_frame(context: Rc<RefCell<SceneContext>>, handler: Box<dyn LoopHandler>) -> Result<(), JsValue> {
    let closure = Closure::wrap(Box::new(move || {
        let projection = &context.borrow().renderer_context.projection;
        let handler = handler.handle_render_loop(projection.canvas_width, projection.canvas_height);
        let sprites = handler.create_sprites(projection.canvas_width, projection.canvas_height);
        context.borrow().renderer_context.renderer.render(projection, &sprites, &context.borrow().renderer_context.atlas);
        request_animation_frame(context.clone(), handler);
    }) as Box<dyn Fn()>);
    let window = web_sys::window().unwrap();
    window.request_animation_frame(closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}
