//! ox∅ fluid simulation — Jos Stam's "Stable Fluids" in Rust/WASM + WebGL.
//! Mouse-interactive 2D Navier-Stokes solver at 60fps.

use crate::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::cell::RefCell;
use std::rc::Rc;

const N: usize = 256;
const SIZE: usize = (N + 2) * (N + 2);
const DT: f32 = 0.1;
const DIFF: f32 = 0.0;     // diffusion rate (0 = no diffusion, looks best)
const VISC: f32 = 0.0;     // viscosity (0 = inviscid)
const GAUSS_SEIDEL_ITERS: usize = 20;

// ── Index macro ───────────────────────────────────────────────────

#[inline(always)]
fn ix(i: usize, j: usize) -> usize { i + (N + 2) * j }

// ── Fluid Solver ──────────────────────────────────────────────────

struct FluidSolver {
    u: Vec<f32>,       // velocity x
    v: Vec<f32>,       // velocity y
    u_prev: Vec<f32>,
    v_prev: Vec<f32>,
    d: Vec<f32>,       // density
    d_prev: Vec<f32>,
}

impl FluidSolver {
    fn new() -> Self {
        Self {
            u: vec![0.0; SIZE], v: vec![0.0; SIZE],
            u_prev: vec![0.0; SIZE], v_prev: vec![0.0; SIZE],
            d: vec![0.0; SIZE], d_prev: vec![0.0; SIZE],
        }
    }

    fn step(&mut self) {
        // Velocity step
        add_source(&mut self.u, &self.u_prev, DT);
        add_source(&mut self.v, &self.v_prev, DT);

        std::mem::swap(&mut self.u, &mut self.u_prev);
        diffuse(1, &mut self.u, &self.u_prev, VISC, DT);
        std::mem::swap(&mut self.v, &mut self.v_prev);
        diffuse(2, &mut self.v, &self.v_prev, VISC, DT);
        project(&mut self.u, &mut self.v, &mut self.u_prev, &mut self.v_prev);

        std::mem::swap(&mut self.u, &mut self.u_prev);
        std::mem::swap(&mut self.v, &mut self.v_prev);
        advect(1, &mut self.u, &self.u_prev, &self.u_prev, &self.v_prev, DT);
        advect(2, &mut self.v, &self.v_prev, &self.u_prev, &self.v_prev, DT);
        project(&mut self.u, &mut self.v, &mut self.u_prev, &mut self.v_prev);

        // Density step
        add_source(&mut self.d, &self.d_prev, DT);
        std::mem::swap(&mut self.d, &mut self.d_prev);
        diffuse(0, &mut self.d, &self.d_prev, DIFF, DT);
        std::mem::swap(&mut self.d, &mut self.d_prev);
        advect(0, &mut self.d, &self.d_prev, &self.u, &self.v, DT);

        // Fade density slightly for visual effect
        for val in self.d.iter_mut() {
            *val *= 0.995;
        }

        // Clear prev for next frame's input
        self.u_prev.fill(0.0);
        self.v_prev.fill(0.0);
        self.d_prev.fill(0.0);
    }

    fn density_pixels(&self) -> Vec<u8> {
        let mut pixels = vec![0u8; N * N * 4]; // RGBA
        for j in 1..=N {
            for i in 1..=N {
                let d = self.d[ix(i, j)].clamp(0.0, 1.0);
                let idx = ((j - 1) * N + (i - 1)) * 4;
                // Blue-cyan-white palette
                let r = (d * d * 80.0).min(255.0) as u8;
                let g = (d * 180.0).min(255.0) as u8;
                let b = (d * 255.0 + 20.0 * d * d).min(255.0) as u8;
                pixels[idx]     = r;
                pixels[idx + 1] = g;
                pixels[idx + 2] = b;
                pixels[idx + 3] = 255;
            }
        }
        pixels
    }
}

// ── Solver functions ──────────────────────────────────────────────

fn add_source(x: &mut [f32], s: &[f32], dt: f32) {
    for i in 0..SIZE { x[i] += dt * s[i]; }
}

fn set_boundary(b: i32, x: &mut [f32]) {
    for i in 1..=N {
        x[ix(0,     i)] = if b == 1 { -x[ix(1, i)] } else { x[ix(1, i)] };
        x[ix(N + 1, i)] = if b == 1 { -x[ix(N, i)] } else { x[ix(N, i)] };
        x[ix(i,     0)] = if b == 2 { -x[ix(i, 1)] } else { x[ix(i, 1)] };
        x[ix(i, N + 1)] = if b == 2 { -x[ix(i, N)] } else { x[ix(i, N)] };
    }
    x[ix(0,     0    )] = 0.5 * (x[ix(1, 0)]     + x[ix(0, 1)]);
    x[ix(0,     N + 1)] = 0.5 * (x[ix(1, N + 1)] + x[ix(0, N)]);
    x[ix(N + 1, 0    )] = 0.5 * (x[ix(N, 0)]     + x[ix(N + 1, 1)]);
    x[ix(N + 1, N + 1)] = 0.5 * (x[ix(N, N + 1)] + x[ix(N + 1, N)]);
}

fn diffuse(b: i32, x: &mut [f32], x0: &[f32], diff: f32, dt: f32) {
    let a = dt * diff * (N * N) as f32;
    if a == 0.0 {
        x.copy_from_slice(x0);
        return;
    }
    let denom = 1.0 + 4.0 * a;
    for _ in 0..GAUSS_SEIDEL_ITERS {
        for j in 1..=N {
            for i in 1..=N {
                x[ix(i, j)] = (x0[ix(i, j)] + a * (
                    x[ix(i - 1, j)] + x[ix(i + 1, j)] +
                    x[ix(i, j - 1)] + x[ix(i, j + 1)]
                )) / denom;
            }
        }
        set_boundary(b, x);
    }
}

fn advect(b: i32, d: &mut [f32], d0: &[f32], u: &[f32], v: &[f32], dt: f32) {
    let dt0 = dt * N as f32;
    for j in 1..=N {
        for i in 1..=N {
            let mut x = i as f32 - dt0 * u[ix(i, j)];
            let mut y = j as f32 - dt0 * v[ix(i, j)];
            x = x.clamp(0.5, N as f32 + 0.5);
            y = y.clamp(0.5, N as f32 + 0.5);
            let i0 = x as usize; let i1 = i0 + 1;
            let j0 = y as usize; let j1 = j0 + 1;
            let s1 = x - i0 as f32; let s0 = 1.0 - s1;
            let t1 = y - j0 as f32; let t0 = 1.0 - t1;
            d[ix(i, j)] = s0 * (t0 * d0[ix(i0, j0)] + t1 * d0[ix(i0, j1)])
                        + s1 * (t0 * d0[ix(i1, j0)] + t1 * d0[ix(i1, j1)]);
        }
    }
    set_boundary(b, d);
}

fn project(u: &mut [f32], v: &mut [f32], p: &mut [f32], div: &mut [f32]) {
    let h = 1.0 / N as f32;
    for j in 1..=N {
        for i in 1..=N {
            div[ix(i, j)] = -0.5 * h * (
                u[ix(i + 1, j)] - u[ix(i - 1, j)] +
                v[ix(i, j + 1)] - v[ix(i, j - 1)]
            );
            p[ix(i, j)] = 0.0;
        }
    }
    set_boundary(0, div);
    set_boundary(0, p);

    for _ in 0..GAUSS_SEIDEL_ITERS {
        for j in 1..=N {
            for i in 1..=N {
                p[ix(i, j)] = (div[ix(i, j)]
                    + p[ix(i - 1, j)] + p[ix(i + 1, j)]
                    + p[ix(i, j - 1)] + p[ix(i, j + 1)]) / 4.0;
            }
        }
        set_boundary(0, p);
    }

    for j in 1..=N {
        for i in 1..=N {
            u[ix(i, j)] -= 0.5 * N as f32 * (p[ix(i + 1, j)] - p[ix(i - 1, j)]);
            v[ix(i, j)] -= 0.5 * N as f32 * (p[ix(i, j + 1)] - p[ix(i, j - 1)]);
        }
    }
    set_boundary(1, u);
    set_boundary(2, v);
}

// ── Mouse Input ───────────────────────────────────────────────────

struct Input {
    x: f32,
    y: f32,
    prev_x: f32,
    prev_y: f32,
    active: bool,
}

impl Input {
    fn new() -> Self {
        Self { x: 0.0, y: 0.0, prev_x: 0.0, prev_y: 0.0, active: false }
    }
}

fn apply_input(solver: &mut FluidSolver, input: &Input) {
    if !input.active { return; }
    let dx = input.x - input.prev_x;
    let dy = input.y - input.prev_y;
    let radius: i32 = 6;
    let r2 = (radius * radius) as f32;

    for di in -radius..=radius {
        for dj in -radius..=radius {
            let gi = input.x as i32 + di;
            let gj = input.y as i32 + dj;
            if gi < 1 || gi > N as i32 || gj < 1 || gj > N as i32 { continue; }
            let dist2 = (di * di + dj * dj) as f32;
            let w = (-dist2 / (2.0 * r2)).exp();
            let idx = ix(gi as usize, gj as usize);
            solver.u_prev[idx] += dx * w * 8.0;
            solver.v_prev[idx] += dy * w * 8.0;
            solver.d_prev[idx] += w * 1.0;
        }
    }
}

// ── WebGL Renderer ────────────────────────────────────────────────

type GL = web_sys::WebGlRenderingContext;

fn setup_webgl(canvas: &web_sys::HtmlCanvasElement) -> (GL, web_sys::WebGlTexture) {
    let gl: GL = canvas
        .get_context("webgl").unwrap().unwrap()
        .dyn_into().unwrap();

    // Shaders
    let vs_src = "attribute vec2 a_pos; attribute vec2 a_uv; varying vec2 v_uv;
        void main() { gl_Position = vec4(a_pos, 0.0, 1.0); v_uv = a_uv; }";
    let fs_src = "precision mediump float; varying vec2 v_uv; uniform sampler2D u_tex;
        void main() { gl_FragColor = texture2D(u_tex, v_uv); }";

    let vs = compile_shader(&gl, GL::VERTEX_SHADER, vs_src);
    let fs = compile_shader(&gl, GL::FRAGMENT_SHADER, fs_src);
    let prog = gl.create_program().unwrap();
    gl.attach_shader(&prog, &vs);
    gl.attach_shader(&prog, &fs);
    gl.link_program(&prog);
    gl.use_program(Some(&prog));

    // Fullscreen quad
    let verts: [f32; 16] = [
        -1.0, -1.0,  0.0, 1.0,
         1.0, -1.0,  1.0, 1.0,
        -1.0,  1.0,  0.0, 0.0,
         1.0,  1.0,  1.0, 0.0,
    ];
    let buf = gl.create_buffer().unwrap();
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buf));
    unsafe {
        let view = js_sys::Float32Array::view(&verts);
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &view, GL::STATIC_DRAW);
    }
    let a_pos = gl.get_attrib_location(&prog, "a_pos") as u32;
    let a_uv = gl.get_attrib_location(&prog, "a_uv") as u32;
    gl.enable_vertex_attrib_array(a_pos);
    gl.enable_vertex_attrib_array(a_uv);
    gl.vertex_attrib_pointer_with_i32(a_pos, 2, GL::FLOAT, false, 16, 0);
    gl.vertex_attrib_pointer_with_i32(a_uv, 2, GL::FLOAT, false, 16, 8);

    // Texture
    let tex = gl.create_texture().unwrap();
    gl.bind_texture(GL::TEXTURE_2D, Some(&tex));
    gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32);
    gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32);
    gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
    gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);

    gl.clear_color(0.0, 0.0, 0.0, 1.0);

    (gl, tex)
}

fn compile_shader(gl: &GL, shader_type: u32, source: &str) -> web_sys::WebGlShader {
    let shader = gl.create_shader(shader_type).unwrap();
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);
    shader
}

fn render(gl: &GL, tex: &web_sys::WebGlTexture, pixels: &[u8]) {
    gl.bind_texture(GL::TEXTURE_2D, Some(tex));
    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
        GL::TEXTURE_2D, 0, GL::RGBA as i32,
        N as i32, N as i32, 0,
        GL::RGBA, GL::UNSIGNED_BYTE, Some(pixels),
    ).unwrap();
    gl.clear(GL::COLOR_BUFFER_BIT);
    gl.draw_arrays(GL::TRIANGLE_STRIP, 0, 4);
}

// ── Entry Point ───────────────────────────────────────────────────

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    let doc = document();
    let root = doc.get_element_by_id("app").unwrap();

    // Create canvas
    let canvas = doc.create_element("canvas").unwrap();
    canvas.set_attribute("width", "512").unwrap();
    canvas.set_attribute("height", "512").unwrap();
    canvas.set_attribute("style",
        "width:100%;max-width:512px;aspect-ratio:1;display:block;margin:0 auto;cursor:crosshair;border-radius:0.75rem"
    ).unwrap();
    root.append_child(&canvas).unwrap();

    // Info text
    let info = text_el("p", "ox-text-sm ox-text-center ox-mt-4",
        "Click and drag to inject fluid");
    attr(&info, "style", "color:#9ca0ab");
    root.append_child(&info).unwrap();

    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
    let (gl, tex) = setup_webgl(&canvas);

    // Shared state
    let solver = Rc::new(RefCell::new(FluidSolver::new()));
    let input = Rc::new(RefCell::new(Input::new()));

    // Mouse events
    {
        let input = input.clone();
        let canvas_ref = canvas.clone();
        let cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            let mut inp = input.borrow_mut();
            let rect = canvas_ref.get_bounding_client_rect();
            let scale_x = N as f64 / rect.width();
            let scale_y = N as f64 / rect.height();
            inp.prev_x = inp.x;
            inp.prev_y = inp.y;
            inp.x = ((e.client_x() as f64 - rect.left()) * scale_x).clamp(1.0, N as f64) as f32;
            inp.y = ((e.client_y() as f64 - rect.top()) * scale_y).clamp(1.0, N as f64) as f32;
        }) as Box<dyn Fn(web_sys::MouseEvent)>);
        canvas.add_event_listener_with_callback("mousemove", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }
    {
        let input = input.clone();
        let cb = Closure::wrap(Box::new(move |_: web_sys::MouseEvent| {
            input.borrow_mut().active = true;
        }) as Box<dyn Fn(web_sys::MouseEvent)>);
        canvas.add_event_listener_with_callback("mousedown", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }
    {
        let input = input.clone();
        let cb = Closure::wrap(Box::new(move |_: web_sys::MouseEvent| {
            input.borrow_mut().active = false;
        }) as Box<dyn Fn(web_sys::MouseEvent)>);
        canvas.add_event_listener_with_callback("mouseup", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }

    // Touch events for mobile
    {
        let input = input.clone();
        let canvas_ref = canvas.clone();
        let cb = Closure::wrap(Box::new(move |e: web_sys::Event| {
            e.prevent_default();
            let te: &web_sys::TouchEvent = e.unchecked_ref();
            if let Some(touch) = te.touches().get(0) {
                let mut inp = input.borrow_mut();
                let rect = canvas_ref.get_bounding_client_rect();
                let scale_x = N as f64 / rect.width();
                let scale_y = N as f64 / rect.height();
                inp.prev_x = inp.x;
                inp.prev_y = inp.y;
                inp.x = ((touch.client_x() as f64 - rect.left()) * scale_x).clamp(1.0, N as f64) as f32;
                inp.y = ((touch.client_y() as f64 - rect.top()) * scale_y).clamp(1.0, N as f64) as f32;
                inp.active = true;
            }
        }) as Box<dyn Fn(web_sys::Event)>);
        canvas.add_event_listener_with_callback("touchmove", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }
    {
        let input = input.clone();
        let cb = Closure::wrap(Box::new(move |_: web_sys::Event| {
            input.borrow_mut().active = false;
        }) as Box<dyn Fn(web_sys::Event)>);
        canvas.add_event_listener_with_callback("touchend", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }

    // Animation loop
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        // Apply input
        {
            let inp = input.borrow();
            apply_input(&mut solver.borrow_mut(), &inp);
        }

        // Step simulation
        solver.borrow_mut().step();

        // Render
        let pixels = solver.borrow().density_pixels();
        render(&gl, &tex, &pixels);

        // Request next frame
        web_sys::window().unwrap()
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }) as Box<dyn FnMut()>));

    web_sys::window().unwrap()
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();
}
