//! ox∅ Ray Marcher — real-time 3D SDF rendering in a fragment shader.
//! Morphing fractal landscape with mouse-controlled camera.

use crate::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::cell::RefCell;
use std::rc::Rc;

type GL = web_sys::WebGlRenderingContext;

const VS: &str = "attribute vec2 a_pos;
void main() { gl_Position = vec4(a_pos, 0.0, 1.0); }";

const FS: &str = "precision highp float;
uniform float u_time;
uniform vec2 u_resolution;
uniform vec2 u_mouse;

// Signed distance functions
float sdSphere(vec3 p, float r) { return length(p) - r; }
float sdBox(vec3 p, vec3 b) { vec3 d = abs(p) - b; return min(max(d.x,max(d.y,d.z)),0.0) + length(max(d,0.0)); }
float sdTorus(vec3 p, vec2 t) { vec2 q = vec2(length(p.xz)-t.x,p.y); return length(q)-t.y; }

// Smooth min for organic blending
float smin(float a, float b, float k) {
    float h = clamp(0.5 + 0.5*(b-a)/k, 0.0, 1.0);
    return mix(b, a, h) - k*h*(1.0-h);
}

// Rotation
mat2 rot(float a) { float c=cos(a),s=sin(a); return mat2(c,-s,s,c); }

// Scene SDF — morphing shapes
float scene(vec3 p) {
    float t = u_time * 0.5;

    // Ground plane with sine waves
    float ground = p.y + 1.5 + 0.3 * sin(p.x * 1.5 + t) * cos(p.z * 1.2 + t * 0.7);

    // Morphing central object
    vec3 q = p;
    q.xz *= rot(t * 0.4);
    q.xy *= rot(t * 0.3);

    float morph = mix(
        sdBox(q, vec3(0.8)),
        sdTorus(q, vec2(1.0, 0.35)),
        0.5 + 0.5 * sin(t * 0.8)
    );

    // Orbiting spheres
    float spheres = 1e10;
    for (int i = 0; i < 5; i++) {
        float a = float(i) * 1.2566 + t * 0.6;  // 2π/5
        vec3 sp = vec3(cos(a) * 2.5, sin(t + float(i)) * 0.5, sin(a) * 2.5);
        float r = 0.3 + 0.1 * sin(t * 2.0 + float(i));
        spheres = min(spheres, sdSphere(p - sp, r));
    }

    // Blend everything
    float d = smin(morph, spheres, 0.8);
    d = smin(d, ground, 0.5);
    return d;
}

// Normal via gradient
vec3 calcNormal(vec3 p) {
    vec2 e = vec2(0.001, 0.0);
    return normalize(vec3(
        scene(p+e.xyy) - scene(p-e.xyy),
        scene(p+e.yxy) - scene(p-e.yxy),
        scene(p+e.yyx) - scene(p-e.yyx)
    ));
}

void main() {
    vec2 uv = (gl_FragCoord.xy - 0.5 * u_resolution) / u_resolution.y;

    // Camera — mouse controls orbit
    float mx = u_mouse.x * 3.14159;
    float my = u_mouse.y * 1.5 - 0.3;
    vec3 ro = vec3(
        5.0 * sin(mx) * cos(my),
        2.0 + 3.0 * sin(my),
        5.0 * cos(mx) * cos(my)
    );
    vec3 target = vec3(0.0, 0.0, 0.0);
    vec3 fwd = normalize(target - ro);
    vec3 right = normalize(cross(fwd, vec3(0.0, 1.0, 0.0)));
    vec3 up = cross(right, fwd);
    vec3 rd = normalize(fwd * 1.5 + right * uv.x + up * uv.y);

    // Ray march
    float t = 0.0;
    float d;
    vec3 p;
    for (int i = 0; i < 100; i++) {
        p = ro + rd * t;
        d = scene(p);
        if (d < 0.001 || t > 50.0) break;
        t += d;
    }

    // Shading
    vec3 col = vec3(0.02, 0.03, 0.06); // background
    if (t < 50.0) {
        vec3 n = calcNormal(p);
        vec3 light = normalize(vec3(1.0, 2.0, -1.0));
        vec3 light2 = normalize(vec3(-1.0, 0.5, 1.0));

        // Diffuse
        float diff = max(dot(n, light), 0.0);
        float diff2 = max(dot(n, light2), 0.0);

        // Specular
        vec3 ref1 = reflect(-light, n);
        float spec = pow(max(dot(ref1, normalize(ro - p)), 0.0), 32.0);

        // Color based on position + normal
        vec3 baseColor = 0.5 + 0.5 * cos(vec3(0.0, 2.0, 4.0) + p.y * 0.5 + u_time * 0.2);

        col = baseColor * (0.15 + diff * 0.6 + diff2 * 0.2);
        col += vec3(1.0, 0.9, 0.8) * spec * 0.5;

        // Fog
        float fog = 1.0 - exp(-t * 0.06);
        col = mix(col, vec3(0.02, 0.03, 0.06), fog);
    }

    // Gamma
    col = pow(col, vec3(0.4545));
    gl_FragColor = vec4(col, 1.0);
}";

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    let doc = document();
    let root = doc.get_element_by_id("app").unwrap();

    let container = el("div", "");
    attr(&container, "style",
        "display:flex;flex-direction:column;align-items:center;padding:2rem");

    let title = text_el("h1", "ox-h3 ox-font-display ox-text-center ox-mb-2", "Ray Marcher");
    let sub = text_el("p", "ox-text-center ox-mb-4", "Move mouse to orbit camera. Real-time SDF rendering in GLSL.");
    attr(&sub, "style", "color:#9ca0ab;font-size:0.875rem");

    let canvas = doc.create_element("canvas").unwrap();
    canvas.set_attribute("width", "800").unwrap();
    canvas.set_attribute("height", "600").unwrap();
    canvas.set_attribute("style",
        "width:100%;max-width:800px;aspect-ratio:4/3;display:block;\
         border-radius:0.75rem;border:1px solid #2e3140;cursor:crosshair"
    ).unwrap();

    append(&container, &[&title, &sub, &canvas]);
    root.append_child(&container).unwrap();

    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
    let gl: GL = canvas.get_context("webgl").unwrap().unwrap().dyn_into().unwrap();

    // Compile shaders
    let vs = compile(&gl, GL::VERTEX_SHADER, VS);
    let fs = compile(&gl, GL::FRAGMENT_SHADER, FS);
    let prog = gl.create_program().unwrap();
    gl.attach_shader(&prog, &vs);
    gl.attach_shader(&prog, &fs);
    gl.link_program(&prog);
    gl.use_program(Some(&prog));

    // Fullscreen quad
    let verts: [f32; 8] = [-1.0,-1.0, 1.0,-1.0, -1.0,1.0, 1.0,1.0];
    let buf = gl.create_buffer().unwrap();
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buf));
    unsafe {
        let view = js_sys::Float32Array::view(&verts);
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &view, GL::STATIC_DRAW);
    }
    let a_pos = gl.get_attrib_location(&prog, "a_pos") as u32;
    gl.enable_vertex_attrib_array(a_pos);
    gl.vertex_attrib_pointer_with_i32(a_pos, 2, GL::FLOAT, false, 0, 0);

    // Uniforms
    let u_time = gl.get_uniform_location(&prog, "u_time");
    let u_resolution = gl.get_uniform_location(&prog, "u_resolution");
    let u_mouse = gl.get_uniform_location(&prog, "u_mouse");

    gl.uniform2f(u_resolution.as_ref(), 800.0, 600.0);

    // Mouse state
    let mouse = Rc::new(RefCell::new((0.5f32, 0.5f32)));
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

    // Animation loop
    let start = js_sys::Date::now();
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let t = ((js_sys::Date::now() - start) / 1000.0) as f32;
        let m = mouse.borrow();

        gl.uniform1f(u_time.as_ref(), t);
        gl.uniform2f(u_mouse.as_ref(), m.0, m.1);
        gl.draw_arrays(GL::TRIANGLE_STRIP, 0, 4);

        web_sys::window().unwrap()
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }) as Box<dyn FnMut()>));

    web_sys::window().unwrap()
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();
}

fn compile(gl: &GL, t: u32, src: &str) -> web_sys::WebGlShader {
    let s = gl.create_shader(t).unwrap();
    gl.shader_source(&s, src);
    gl.compile_shader(&s);
    s
}
