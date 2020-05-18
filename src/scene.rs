use crate::renderer::{Renderer, TextureAtlas, Projection, Sprite};
use crate::geom::Point;
use crate::logger::{log_info, log_error, log_debug};
use wasm_bindgen::JsValue;
use std::cell::{RefMut, RefCell};
use std::rc::Rc;
use crate::scene::renderer::{Renderer, TextureAtlas, Projection, Sprite};
use crate::scene::geom::Point;

mod scene_loader;
mod renderer;

trait LoopHandler {
    fn handle_render_loop(&mut self) -> Vec<Sprite>;
}

pub struct RendererContext {
    renderer: Renderer,
    atlas: TextureAtlas,
    projection: Projection,
}

pub struct SceneContext {
    renderer_context: RendererContext,
    mouse_pos: Option<Point>,
    loop_handler: *dyn LoopHandler,
}

pub fn init(window: &web_sys::Window, loop_handler: *dyn LoopHandler) -> Result<(), (JsValue)> {
    let mut context = SceneContext {
        renderer_context: create_renderer(window)?,
        mouse_pos: None,
        loop_handler
    };
    let context_rc = Rc::new(RefCell::new(context));
    request_animation_frame(window,context_rc.clone())?;

    // resize
    {
        let context_rc = context_rc.clone();
        let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
            match {create_renderer(window)} {
                Ok(renderer_context) => {
                    context_rc.clone().borrow_mut().renderer_context = renderer_context;
                },
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
            mouse_move_handler(context_rc.borrow_mut(), e);
        }) as Box<dyn Fn(MouseEvent)>);
        window.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref());
        closure.forget();
    }
    {
        let context_rc = context_rc.clone();
        let closure = Closure::wrap(Box::new(move |e: TouchEvent| {
            touch_move_handler(context_rc.borrow_mut(), e);
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
        Some(Point { x: e.client_x() as f32, y : e.client_y() as f32 })
    } else { None };
    log_debug(format!("Mouse: {:?}", &context.mouse_pos).as_str());
}

fn touch_move_handler(mut context: RefMut<SceneContext>, e: TouchEvent) {
    context.mouse_pos = match e.touches().get(0) {
        Some(t) => Some(Point { x: t.client_x() as f32, y : t.client_y() as f32 }),
        None => None
    };
    log_debug(format!("Mouse: {:?}", &context.mouse_pos).as_str());
}

fn create_renderer(window: &web_sys::Window, ) -> Result<RendererContext, JsValue> {
    //let window = web_sys::window().unwrap();
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
    let atlas = TextureAtlas::empty();
    let projection = Projection::create(width, height);
    Ok(RendererContext { renderer, atlas, projection })
}

fn request_animation_frame(window: &web_sys::Window, context: Rc<RefCell<SceneContext>>) -> Result<(), JsValue> {
    let closure = Closure::wrap(Box::new(move || {
        render_loop(context.borrow_mut());
        request_animation_frame(window, context.clone());
    }) as Box<dyn Fn()>);
    //let window = web_sys::window().unwrap();
    window.request_animation_frame(closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

fn render_loop(mut context: RefMut<SceneContext>) {
    let sprites = context.loop_handler.handle_render_loop();
    context.renderer_context.renderer.render(&context.renderer_context.projection,
                                             &sprites, &context.renderer_context.atlas);
}