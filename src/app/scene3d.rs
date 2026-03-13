//! ox∅ 3D Scene — vertex-based WebGL with lighting, camera, and multiple objects.
//! Rust computes meshes + matrices, GPU renders.

use crate::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::cell::RefCell;
use std::rc::Rc;

type GL = web_sys::WebGlRenderingContext;

// ── Shaders ───────────────────────────────────────────────────────

const VS: &str = "
attribute vec3 a_pos;
attribute vec3 a_normal;
attribute vec3 a_color;
uniform mat4 u_proj;
uniform mat4 u_view;
uniform mat4 u_model;
varying vec3 v_normal;
varying vec3 v_color;
varying vec3 v_pos;
void main() {
    vec4 worldPos = u_model * vec4(a_pos, 1.0);
    v_pos = worldPos.xyz;
    v_normal = mat3(u_model) * a_normal;
    v_color = a_color;
    gl_Position = u_proj * u_view * worldPos;
}";

const FS: &str = "precision mediump float;
varying vec3 v_normal;
varying vec3 v_color;
varying vec3 v_pos;
uniform vec3 u_light;
uniform vec3 u_eye;
void main() {
    vec3 n = normalize(v_normal);
    vec3 l = normalize(u_light - v_pos);
    vec3 v = normalize(u_eye - v_pos);
    vec3 h = normalize(l + v);

    float ambient = 0.15;
    float diff = max(dot(n, l), 0.0) * 0.7;
    float spec = pow(max(dot(n, h), 0.0), 64.0) * 0.4;

    vec3 col = v_color * (ambient + diff) + vec3(1.0) * spec;
    col = pow(col, vec3(0.4545)); // gamma
    gl_FragColor = vec4(col, 1.0);
}";

// ── Math ──────────────────────────────────────────────────────────

type Mat4 = [f32; 16];

fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    let f = 1.0 / (fov * 0.5).tan();
    let nf = 1.0 / (near - far);
    [
        f / aspect, 0.0, 0.0, 0.0,
        0.0, f, 0.0, 0.0,
        0.0, 0.0, (far + near) * nf, -1.0,
        0.0, 0.0, 2.0 * far * near * nf, 0.0,
    ]
}

fn look_at(eye: [f32; 3], center: [f32; 3], up: [f32; 3]) -> Mat4 {
    let f = normalize3(sub3(center, eye));
    let s = normalize3(cross3(f, up));
    let u = cross3(s, f);
    [
        s[0], u[0], -f[0], 0.0,
        s[1], u[1], -f[1], 0.0,
        s[2], u[2], -f[2], 0.0,
        -dot3(s, eye), -dot3(u, eye), dot3(f, eye), 1.0,
    ]
}

fn model_matrix(tx: f32, ty: f32, tz: f32, angle: f32, ax: f32, ay: f32, az: f32, scale: f32) -> Mat4 {
    let c = angle.cos(); let s = angle.sin(); let t = 1.0 - c;
    let len = (ax*ax + ay*ay + az*az).sqrt();
    let (x, y, z) = (ax/len, ay/len, az/len);
    [
        (t*x*x + c) * scale, (t*x*y + s*z) * scale, (t*x*z - s*y) * scale, 0.0,
        (t*x*y - s*z) * scale, (t*y*y + c) * scale, (t*y*z + s*x) * scale, 0.0,
        (t*x*z + s*y) * scale, (t*y*z - s*x) * scale, (t*z*z + c) * scale, 0.0,
        tx, ty, tz, 1.0,
    ]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[0]-b[0], a[1]-b[1], a[2]-b[2]] }
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]] }
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 { a[0]*b[0] + a[1]*b[1] + a[2]*b[2] }
fn normalize3(v: [f32; 3]) -> [f32; 3] { let l = dot3(v, v).sqrt(); [v[0]/l, v[1]/l, v[2]/l] }

// ── Mesh Generation ───────────────────────────────────────────────

struct Mesh {
    positions: Vec<f32>,
    normals: Vec<f32>,
    colors: Vec<f32>,
    indices: Vec<u16>,
}

fn generate_sphere(subdivisions: usize, r: f32, g: f32, b: f32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();
    let stacks = subdivisions;
    let slices = subdivisions;

    for i in 0..=stacks {
        let phi = std::f32::consts::PI * i as f32 / stacks as f32;
        for j in 0..=slices {
            let theta = 2.0 * std::f32::consts::PI * j as f32 / slices as f32;
            let x = phi.sin() * theta.cos();
            let y = phi.cos();
            let z = phi.sin() * theta.sin();
            positions.extend_from_slice(&[x, y, z]);
            normals.extend_from_slice(&[x, y, z]);
            colors.extend_from_slice(&[r, g, b]);
        }
    }
    for i in 0..stacks {
        for j in 0..slices {
            let a = (i * (slices + 1) + j) as u16;
            let b = a + (slices + 1) as u16;
            indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    Mesh { positions, normals, colors, indices }
}

fn generate_cube(r: f32, g: f32, b: f32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    let faces: [([f32; 3], [[f32; 3]; 4]); 6] = [
        ([0.0,0.0,1.0],  [[-1.0,-1.0,1.0],[1.0,-1.0,1.0],[1.0,1.0,1.0],[-1.0,1.0,1.0]]),
        ([0.0,0.0,-1.0], [[-1.0,-1.0,-1.0],[-1.0,1.0,-1.0],[1.0,1.0,-1.0],[1.0,-1.0,-1.0]]),
        ([0.0,1.0,0.0],  [[-1.0,1.0,-1.0],[-1.0,1.0,1.0],[1.0,1.0,1.0],[1.0,1.0,-1.0]]),
        ([0.0,-1.0,0.0], [[-1.0,-1.0,-1.0],[1.0,-1.0,-1.0],[1.0,-1.0,1.0],[-1.0,-1.0,1.0]]),
        ([1.0,0.0,0.0],  [[1.0,-1.0,-1.0],[1.0,1.0,-1.0],[1.0,1.0,1.0],[1.0,-1.0,1.0]]),
        ([-1.0,0.0,0.0], [[-1.0,-1.0,-1.0],[-1.0,-1.0,1.0],[-1.0,1.0,1.0],[-1.0,1.0,-1.0]]),
    ];

    for (n, verts) in &faces {
        let base = (positions.len() / 3) as u16;
        for v in verts {
            positions.extend_from_slice(v);
            normals.extend_from_slice(n);
            colors.extend_from_slice(&[r, g, b]);
        }
        indices.extend_from_slice(&[base, base+1, base+2, base, base+2, base+3]);
    }
    Mesh { positions, normals, colors, indices }
}

fn generate_torus(major: f32, minor: f32, rings: usize, sides: usize, r: f32, g: f32, b: f32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=rings {
        let u = 2.0 * std::f32::consts::PI * i as f32 / rings as f32;
        for j in 0..=sides {
            let v = 2.0 * std::f32::consts::PI * j as f32 / sides as f32;
            let x = (major + minor * v.cos()) * u.cos();
            let y = minor * v.sin();
            let z = (major + minor * v.cos()) * u.sin();
            let nx = v.cos() * u.cos();
            let ny = v.sin();
            let nz = v.cos() * u.sin();
            positions.extend_from_slice(&[x, y, z]);
            normals.extend_from_slice(&[nx, ny, nz]);
            colors.extend_from_slice(&[r, g, b]);
        }
    }
    for i in 0..rings {
        for j in 0..sides {
            let a = (i * (sides + 1) + j) as u16;
            let b = a + (sides + 1) as u16;
            indices.extend_from_slice(&[a, b, a+1, a+1, b, b+1]);
        }
    }
    Mesh { positions, normals, colors, indices }
}

// ── WebGL Helpers ─────────────────────────────────────────────────

struct GpuMesh {
    index_count: i32,
    _pos_buf: web_sys::WebGlBuffer,
    _norm_buf: web_sys::WebGlBuffer,
    _col_buf: web_sys::WebGlBuffer,
    idx_buf: web_sys::WebGlBuffer,
}

fn upload_mesh(gl: &GL, prog: &web_sys::WebGlProgram, mesh: &Mesh) -> GpuMesh {
    let pos_buf = make_buffer(gl, &mesh.positions);
    let norm_buf = make_buffer(gl, &mesh.normals);
    let col_buf = make_buffer(gl, &mesh.colors);

    let idx_buf = gl.create_buffer().unwrap();
    gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&idx_buf));
    unsafe {
        let view = js_sys::Uint16Array::view(&mesh.indices);
        gl.buffer_data_with_array_buffer_view(GL::ELEMENT_ARRAY_BUFFER, &view, GL::STATIC_DRAW);
    }

    GpuMesh {
        index_count: mesh.indices.len() as i32,
        _pos_buf: pos_buf,
        _norm_buf: norm_buf,
        _col_buf: col_buf,
        idx_buf,
    }
}

fn bind_mesh(gl: &GL, prog: &web_sys::WebGlProgram, gpu: &GpuMesh, mesh: &Mesh) {
    bind_attr(gl, prog, "a_pos", &gpu._pos_buf, 3);
    bind_attr(gl, prog, "a_normal", &gpu._norm_buf, 3);
    bind_attr(gl, prog, "a_color", &gpu._col_buf, 3);
    gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&gpu.idx_buf));
}

fn bind_attr(gl: &GL, prog: &web_sys::WebGlProgram, name: &str, buf: &web_sys::WebGlBuffer, size: i32) {
    let loc = gl.get_attrib_location(prog, name);
    if loc < 0 { return; }
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(buf));
    gl.enable_vertex_attrib_array(loc as u32);
    gl.vertex_attrib_pointer_with_i32(loc as u32, size, GL::FLOAT, false, 0, 0);
}

fn make_buffer(gl: &GL, data: &[f32]) -> web_sys::WebGlBuffer {
    let buf = gl.create_buffer().unwrap();
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buf));
    unsafe {
        let view = js_sys::Float32Array::view(data);
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &view, GL::STATIC_DRAW);
    }
    buf
}

fn set_mat4(gl: &GL, loc: Option<&web_sys::WebGlUniformLocation>, m: &Mat4) {
    gl.uniform_matrix4fv_with_f32_array(loc, false, m);
}

fn compile(gl: &GL, t: u32, src: &str) -> web_sys::WebGlShader {
    let s = gl.create_shader(t).unwrap();
    gl.shader_source(&s, src);
    gl.compile_shader(&s);
    s
}

// ── Entry Point ───────────────────────────────────────────────────

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    let doc = document();
    let root = doc.get_element_by_id("app").unwrap();

    let container = el("div", "");
    attr(&container, "style",
        "display:flex;flex-direction:column;align-items:center;padding:2rem");

    let title = text_el("h1", "ox-h3 ox-font-display ox-text-center ox-mb-2", "3D Scene");
    let sub = text_el("p", "ox-text-center ox-mb-4",
        "Vertex-based WebGL. Rust builds meshes + matrices, GPU renders with Phong lighting.");
    attr(&sub, "style", "color:#9ca0ab;font-size:0.875rem");

    let canvas = doc.create_element("canvas").unwrap();
    canvas.set_attribute("width", "800").unwrap();
    canvas.set_attribute("height", "600").unwrap();
    canvas.set_attribute("style",
        "width:100%;max-width:800px;aspect-ratio:4/3;display:block;\
         border-radius:0.75rem;border:1px solid #2e3140;cursor:grab"
    ).unwrap();

    append(&container, &[&title, &sub, &canvas]);
    root.append_child(&container).unwrap();

    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
    let gl: GL = canvas.get_context("webgl").unwrap().unwrap().dyn_into().unwrap();

    gl.enable(GL::DEPTH_TEST);
    gl.enable(GL::CULL_FACE);
    gl.clear_color(0.04, 0.05, 0.09, 1.0);

    // Shaders
    let vs = compile(&gl, GL::VERTEX_SHADER, VS);
    let fs = compile(&gl, GL::FRAGMENT_SHADER, FS);
    let prog = gl.create_program().unwrap();
    gl.attach_shader(&prog, &vs);
    gl.attach_shader(&prog, &fs);
    gl.link_program(&prog);
    gl.use_program(Some(&prog));

    // Uniforms
    let u_proj = gl.get_uniform_location(&prog, "u_proj");
    let u_view = gl.get_uniform_location(&prog, "u_view");
    let u_model = gl.get_uniform_location(&prog, "u_model");
    let u_light = gl.get_uniform_location(&prog, "u_light");
    let u_eye = gl.get_uniform_location(&prog, "u_eye");

    let proj = perspective(0.8, 800.0 / 600.0, 0.1, 100.0);
    set_mat4(&gl, u_proj.as_ref(), &proj);
    gl.uniform3f(u_light.as_ref(), 4.0, 6.0, 4.0);

    // Generate meshes
    let sphere_mesh = generate_sphere(24, 0.3, 0.6, 1.0);
    let cube_mesh = generate_cube(1.0, 0.4, 0.3);
    let torus_mesh = generate_torus(1.2, 0.4, 32, 16, 0.2, 0.9, 0.5);

    let sphere_gpu = upload_mesh(&gl, &prog, &sphere_mesh);
    let cube_gpu = upload_mesh(&gl, &prog, &cube_mesh);
    let torus_gpu = upload_mesh(&gl, &prog, &torus_mesh);

    // Mouse orbit
    let mouse = Rc::new(RefCell::new((0.5f32, 0.3f32)));
    {
        let mouse = mouse.clone();
        let canvas_ref = canvas.clone();
        let cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            let rect = canvas_ref.get_bounding_client_rect();
            let x = ((e.client_x() as f64 - rect.left()) / rect.width()) as f32;
            let y = ((e.client_y() as f64 - rect.top()) / rect.height()) as f32;
            *mouse.borrow_mut() = (x, y);
        }) as Box<dyn Fn(web_sys::MouseEvent)>);
        canvas.add_event_listener_with_callback("mousemove", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }

    // Animation
    let start = js_sys::Date::now();
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let t = ((js_sys::Date::now() - start) / 1000.0) as f32;
        let m = mouse.borrow();

        // Camera orbit
        let angle_h = m.0 * std::f32::consts::TAU;
        let angle_v = (m.1 - 0.5) * 2.0;
        let dist = 8.0;
        let eye = [
            dist * angle_h.sin() * (1.0 - angle_v.abs() * 0.8),
            2.0 + dist * angle_v * 0.5,
            dist * angle_h.cos() * (1.0 - angle_v.abs() * 0.8),
        ];
        let view = look_at(eye, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        set_mat4(&gl, u_view.as_ref(), &view);
        gl.uniform3f(u_eye.as_ref(), eye[0], eye[1], eye[2]);

        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

        // Draw torus — center, rotating
        let m1 = model_matrix(0.0, 0.0, 0.0, t * 0.5, 0.0, 1.0, 0.3, 1.0);
        set_mat4(&gl, u_model.as_ref(), &m1);
        bind_mesh(&gl, &prog, &torus_gpu, &torus_mesh);
        gl.draw_elements_with_i32(GL::TRIANGLES, torus_gpu.index_count, GL::UNSIGNED_SHORT, 0);

        // Draw cube — orbiting
        let cx = 3.0 * (t * 0.7).cos();
        let cz = 3.0 * (t * 0.7).sin();
        let m2 = model_matrix(cx, 0.5 * (t * 1.5).sin(), cz, t, 1.0, 1.0, 0.0, 0.6);
        set_mat4(&gl, u_model.as_ref(), &m2);
        bind_mesh(&gl, &prog, &cube_gpu, &cube_mesh);
        gl.draw_elements_with_i32(GL::TRIANGLES, cube_gpu.index_count, GL::UNSIGNED_SHORT, 0);

        // Draw spheres — scattered
        for i in 0..5 {
            let a = i as f32 * 1.2566 + t * 0.3;
            let r = 2.5 + 0.5 * (t * 0.4 + i as f32).sin();
            let sx = r * a.cos();
            let sz = r * a.sin();
            let sy = (t * 0.8 + i as f32 * 1.5).sin() * 0.8;
            let m3 = model_matrix(sx, sy, sz, t + i as f32, 0.0, 1.0, 0.0, 0.4);
            set_mat4(&gl, u_model.as_ref(), &m3);
            bind_mesh(&gl, &prog, &sphere_gpu, &sphere_mesh);
            gl.draw_elements_with_i32(GL::TRIANGLES, sphere_gpu.index_count, GL::UNSIGNED_SHORT, 0);
        }

        web_sys::window().unwrap()
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }) as Box<dyn FnMut()>));

    web_sys::window().unwrap()
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();
}
