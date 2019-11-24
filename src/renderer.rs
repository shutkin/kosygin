use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d, Document, ImageBitmap,
              WebGlBuffer, WebGlProgram, WebGlRenderingContext, WebGlShader, WebGlTexture};

use crate::logger::{log_debug, log_info};
use crate::geom::Point;

pub struct Renderer {
    gl: WebGlRenderingContext,
    vertices_buffer: WebGlBuffer,
    indices_buffer: WebGlBuffer,
    program: WebGlProgram,
}

pub struct Projection {
    pub canvas_width: u32,
    pub canvas_height: u32,
    matrix: [f32; 9],
}

pub struct TexAtlasItem {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

pub struct TextureAtlas {
    items: Vec<TexAtlasItem>,
    width: u32,
    height: u32,
}

impl TextureAtlas {
    pub fn empty() -> TextureAtlas {
        TextureAtlas { items: Vec::new(), width: 0, height: 0 }
    }
}

pub struct Sprite {
    pub texture: usize,
    pub position: Point,
    pub pivot: Point,
    pub rotation: f32,
    pub width: f32,
    pub height: f32,
    pub alpha: f32,
}

impl Projection {
    pub fn create(canvas_width: u32, canvas_height: u32) -> Projection {
        let matrix: [f32; 9] = [
            2_f32 / canvas_width as f32, 0_f32, 0_f32,
            0_f32, -2_f32 / canvas_height as f32, 0_f32,
            -1_f32, 1_f32, 1_f32
        ];
        Projection { canvas_width, canvas_height, matrix }
    }
}

impl Renderer {
    pub fn init(canvas: &HtmlCanvasElement) -> Result<Renderer, JsValue> {
        let context = canvas.get_context("webgl")?.unwrap();
        let gl: WebGlRenderingContext = context.dyn_into::<WebGlRenderingContext>()?;
        let vertices_buffer = gl.create_buffer().ok_or("failed to create vertices buffer")?;
        let indices_buffer = gl.create_buffer().ok_or("failed to create indices buffer")?;
        let vert_shader = Renderer::compile_shader(&gl, WebGlRenderingContext::VERTEX_SHADER, VERTEX_SHADER)?;
        let frag_shader = Renderer::compile_shader(&gl, WebGlRenderingContext::FRAGMENT_SHADER, FRAGMENT_SHADER)?;
        let program = Renderer::link_program(&gl, &vert_shader, &frag_shader)?;
        gl.use_program(Some(&program));
        gl.disable(WebGlRenderingContext::STENCIL_TEST);
        gl.disable(WebGlRenderingContext::DEPTH_TEST);
        gl.enable(WebGlRenderingContext::BLEND);
        gl.blend_func(WebGlRenderingContext::SRC_ALPHA, WebGlRenderingContext::ONE_MINUS_SRC_ALPHA);
        log_info("Renderer initialized");
        Ok(Renderer { gl, vertices_buffer, indices_buffer, program })
    }

    fn compile_shader(gl: &WebGlRenderingContext, shader_type: u32, source: &str) -> Result<WebGlShader, String> {
        let shader = gl.create_shader(shader_type)
            .ok_or_else(|| String::from("Unable to create shader object"))?;
        gl.shader_source(&shader, source);
        gl.compile_shader(&shader);

        if gl.get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
            .as_bool().unwrap_or(false) {
            Ok(shader)
        } else {
            Err(gl.get_shader_info_log(&shader)
                .unwrap_or_else(|| String::from("Unknown error creating shader")))
        }
    }

    fn link_program(gl: &WebGlRenderingContext, vert_shader: &WebGlShader, frag_shader: &WebGlShader) -> Result<WebGlProgram, String> {
        let program = gl.create_program()
            .ok_or_else(|| String::from("Unable to create shader object"))?;
        gl.attach_shader(&program, vert_shader);
        gl.attach_shader(&program, frag_shader);
        gl.link_program(&program);

        if gl.get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
            .as_bool().unwrap_or(false) {
            Ok(program)
        } else {
            Err(gl.get_program_info_log(&program)
                .unwrap_or_else(|| String::from("Unknown error creating program object")))
        }
    }

    fn update_buffers(&self, sprites: &Vec<Sprite>, atlas: &TextureAtlas) {
        log_debug("Renderer: update buffers");
        let mut vertices: Vec<f32> = Vec::with_capacity(sprites.len() * 20);
        let mut indices: Vec<u16> = Vec::with_capacity(sprites.len() * 6);
        let atlas_width = atlas.width as f32;
        let atlas_height = atlas.height as f32;
        for (i, sprite) in sprites.iter().enumerate() {
            let tex = &atlas.items[sprite.texture];
            let p = sprite.position - sprite.pivot.rotate(sprite.rotation);
            let width_rotated = Point { x: sprite.width, y: 0.0 }.rotate(sprite.rotation);
            let height_rotated = Point { x: 0.0, y: sprite.height }.rotate(sprite.rotation);
            vertices.extend_from_slice(&[
                p.x, p.y, tex.x as f32 / atlas_width, tex.y as f32 / atlas_height, sprite.alpha,
                p.x + width_rotated.x, p.y + width_rotated.y, (tex.x + tex.width) as f32 / atlas_width, tex.y as f32 / atlas_height, sprite.alpha,
                p.x + width_rotated.x + height_rotated.x, p.y + width_rotated.y + height_rotated.y, (tex.x + tex.width) as f32 / atlas_width, (tex.y + tex.height) as f32 / atlas_height, sprite.alpha,
                p.x + height_rotated.x, p.y + height_rotated.y, tex.x as f32 / atlas_width, (tex.y + tex.height) as f32 / atlas_height, sprite.alpha
            ]);
            let n = i as u16 * 4;
            indices.extend_from_slice(&[n, n + 1, n + 2, n, n + 2, n + 3]);
        }

        self.gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&self.vertices_buffer));
        // Note that `Float32Array::view` is somewhat dangerous (hence the
        // `unsafe`!). This is creating a raw view into our module's
        // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
        // (aka do a memory allocation in Rust) it'll cause the buffer to change,
        // causing the `Float32Array` to be invalid.
        //
        // As a result, after `Float32Array::view` we have to be very careful not to
        // do any memory allocations before it's dropped.
        unsafe {
            let vert_array = js_sys::Float32Array::view(&vertices.as_slice());
            self.gl.buffer_data_with_array_buffer_view(WebGlRenderingContext::ARRAY_BUFFER, &vert_array, WebGlRenderingContext::DYNAMIC_DRAW);
        }
        let location = self.gl.get_attrib_location(&self.program, "a_position") as u32;
        self.gl.enable_vertex_attrib_array(location);
        self.gl.vertex_attrib_pointer_with_i32(location, 2, WebGlRenderingContext::FLOAT, false, 20, 0);
        let location = self.gl.get_attrib_location(&self.program, "a_texCoord") as u32;
        self.gl.enable_vertex_attrib_array(location);
        self.gl.vertex_attrib_pointer_with_i32(location, 2, WebGlRenderingContext::FLOAT, true, 20, 8);
        let location = self.gl.get_attrib_location(&self.program, "a_alpha") as u32;
        self.gl.enable_vertex_attrib_array(location);
        self.gl.vertex_attrib_pointer_with_i32(location, 1, WebGlRenderingContext::FLOAT, true, 20, 16);


        self.gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&self.indices_buffer));
        unsafe {
            let ind_array = js_sys::Uint16Array::view(&indices);
            self.gl.buffer_data_with_array_buffer_view(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, &ind_array, WebGlRenderingContext::STATIC_DRAW);
        }
    }

    pub fn create_texture_with_images(&self, document: &Document, images: &Vec<ImageBitmap>) -> Result<TextureAtlas, JsValue> {
        let mut textures: Vec<TexAtlasItem> = Vec::with_capacity(images.len());
        let mut total_height = 0_u32;
        let mut total_width = 0_u32;
        for image in images.iter() {
            let height = image.height() as u32;
            let width = image.width() as u32;
            if total_height < height {
                total_height = height;
            }
            let t = TexAtlasItem { x: total_width, y: 0_u32, width, height };
            log_info(format!("Texture: {} {} {}x{}", &t.x, &t.y, &t.width, &t.height).as_str());
            textures.push(t);
            total_width += width;
        }
        total_height = make_power_2(total_height);
        total_width = make_power_2(total_width);

        let canvas = document.create_element("canvas")?;
        let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>().unwrap();
        canvas.set_width(total_width);
        canvas.set_height(total_height);
        let context = canvas.get_context("2d")?.unwrap();
        let context = context.dyn_into::<CanvasRenderingContext2d>().unwrap();

        for (index, image) in images.iter().enumerate() {
            let tex = &textures[index];
            context.draw_image_with_image_bitmap_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                image, 0_f64, 0_f64, image.width() as f64, image.height() as f64,
                tex.x as f64, tex.y as f64, tex.width as f64, tex.height as f64)?;
        }

        let tex: WebGlTexture = self.gl.create_texture().ok_or("Unable to create texture")?;
        self.gl.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&tex));
        self.gl.tex_image_2d_with_u32_and_u32_and_canvas(WebGlRenderingContext::TEXTURE_2D, 0,
                                                         WebGlRenderingContext::RGBA as i32,
                                                         WebGlRenderingContext::RGBA,
                                                         WebGlRenderingContext::UNSIGNED_BYTE,
                                                         &canvas)?;
        self.gl.tex_parameteri(WebGlRenderingContext::TEXTURE_2D, WebGlRenderingContext::TEXTURE_MAG_FILTER, WebGlRenderingContext::LINEAR as i32);
        self.gl.tex_parameteri(WebGlRenderingContext::TEXTURE_2D, WebGlRenderingContext::TEXTURE_MIN_FILTER, WebGlRenderingContext::LINEAR_MIPMAP_LINEAR as i32);
        self.gl.generate_mipmap(WebGlRenderingContext::TEXTURE_2D);
        Ok(TextureAtlas { items: textures, width: total_width, height: total_height })
    }

    pub fn create_texture_with_canvases(&self, document: &Document, canvases: &Vec<HtmlCanvasElement>) -> Result<TextureAtlas, JsValue> {
        let mut textures: Vec<TexAtlasItem> = Vec::with_capacity(canvases.len());
        let mut total_height = 0_u32;
        let mut total_width = 0_u32;
        for canvas in canvases.iter() {
            let height = canvas.height() as u32;
            let width = canvas.width() as u32;
            if total_height < height {
                total_height = height;
            }
            let t = TexAtlasItem { x: total_width, y: 0_u32, width, height };
            log_info(format!("Texture: {} {} {}x{}", &t.x, &t.y, &t.width, &t.height).as_str());
            textures.push(t);
            total_width += width;
        }
        total_height = make_power_2(total_height);
        total_width = make_power_2(total_width);

        let canvas = document.create_element("canvas")?;
        let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>().unwrap();
        canvas.set_width(total_width);
        canvas.set_height(total_height);
        let context = canvas.get_context("2d")?.unwrap();
        let context = context.dyn_into::<CanvasRenderingContext2d>().unwrap();

        for (index, canvas) in canvases.iter().enumerate() {
            let tex = &textures[index];
            context.draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                canvas, 0_f64, 0_f64, canvas.width() as f64, canvas.height() as f64,
                tex.x as f64, tex.y as f64, tex.width as f64, tex.height as f64)?;
        }

        let tex: WebGlTexture = self.gl.create_texture().ok_or("Unable to create texture")?;
        self.gl.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&tex));
        self.gl.tex_image_2d_with_u32_and_u32_and_canvas(WebGlRenderingContext::TEXTURE_2D, 0,
                                                         WebGlRenderingContext::RGBA as i32,
                                                         WebGlRenderingContext::RGBA,
                                                         WebGlRenderingContext::UNSIGNED_BYTE,
                                                         &canvas)?;
        self.gl.tex_parameteri(WebGlRenderingContext::TEXTURE_2D, WebGlRenderingContext::TEXTURE_MAG_FILTER, WebGlRenderingContext::LINEAR as i32);
        self.gl.tex_parameteri(WebGlRenderingContext::TEXTURE_2D, WebGlRenderingContext::TEXTURE_MIN_FILTER, WebGlRenderingContext::LINEAR_MIPMAP_LINEAR as i32);
        self.gl.generate_mipmap(WebGlRenderingContext::TEXTURE_2D);
        Ok(TextureAtlas { items: textures, width: total_width, height: total_height })
    }

    pub fn render(&self, projection: &Projection, sprites: &Vec<Sprite>, atlas: &TextureAtlas) {
        self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
        self.gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        let location = self.gl.get_uniform_location(&self.program, "u_matrix");
        self.gl.uniform_matrix3fv_with_f32_array(location.as_ref(), false, &projection.matrix);
        self.gl.viewport(0, 0, projection.canvas_width as i32, projection.canvas_height as i32);

        self.update_buffers(sprites, atlas);
        self.gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&self.indices_buffer));
        self.gl.draw_elements_with_i32(WebGlRenderingContext::TRIANGLES, 6 * sprites.len() as i32, WebGlRenderingContext::UNSIGNED_SHORT, 0);
        log_debug("Renderer: render completed");
    }
}

fn make_power_2(v: u32) -> u32 {
    let mut p = 1_u32;
    while p < v {
        p = p * 2;
    }
    p
}

static FRAGMENT_SHADER: &str = "precision mediump float; uniform sampler2D u_image; varying vec2 v_texCoord; varying float v_alpha; \
void main() {gl_FragColor = texture2D(u_image, v_texCoord); gl_FragColor.a = gl_FragColor.a * v_alpha;}";
static VERTEX_SHADER: &str = "attribute vec2 a_position; attribute vec2 a_texCoord; attribute float a_alpha; uniform mat3 u_matrix; varying vec2 v_texCoord; varying float v_alpha; \
void main() {gl_Position = vec4((u_matrix * vec3(a_position, 1)).xy, 0, 1); v_texCoord = a_texCoord; v_alpha = a_alpha;}";