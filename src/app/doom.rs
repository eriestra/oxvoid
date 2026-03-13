//! ox∅ Doom — raycasting FPS engine. WASD + mouse look.

use crate::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::cell::RefCell;
use std::rc::Rc;

const W: usize = 640;
const H: usize = 400;
const MAP_W: usize = 16;
const MAP_H: usize = 16;
const FOV: f64 = std::f64::consts::PI / 3.0; // 60 degrees
const MAX_DIST: f64 = 20.0;
const MOVE_SPEED: f64 = 0.06;
const ROT_SPEED: f64 = 0.04;

// Wall types: 0=empty, 1-5=wall colors
#[rustfmt::skip]
const MAP: [u8; MAP_W * MAP_H] = [
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,2,2,0,0,0,0,0,3,3,3,0,0,1,
    1,0,0,2,0,0,0,0,0,0,0,0,3,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,0,0,0,4,4,4,0,0,0,0,0,0,1,
    1,0,0,0,0,0,4,0,4,0,0,0,0,0,0,1,
    1,0,0,0,0,0,4,0,4,0,0,0,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,5,0,0,0,0,0,0,0,5,0,0,0,1,
    1,0,0,5,0,0,0,0,0,0,0,5,0,0,0,1,
    1,0,0,5,5,5,0,0,0,5,5,5,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
];

fn map_at(x: usize, y: usize) -> u8 {
    if x >= MAP_W || y >= MAP_H { return 1; }
    MAP[y * MAP_W + x]
}

// Wall colors by type
fn wall_color(wall: u8, side: bool) -> (u8, u8, u8) {
    let (r, g, b) = match wall {
        1 => (140, 140, 150),  // gray stone
        2 => (180, 60, 60),    // red brick
        3 => (60, 140, 180),   // blue tech
        4 => (80, 160, 80),    // green moss
        5 => (180, 140, 60),   // gold
        _ => (100, 100, 100),
    };
    // Darken one side for depth
    if side { (r * 3/4, g * 3/4, b * 3/4) } else { (r, g, b) }
}

// ── Player ────────────────────────────────────────────────────────

struct Player {
    x: f64,
    y: f64,
    angle: f64,
}

// ── Input ─────────────────────────────────────────────────────────

struct Input {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    strafe_left: bool,
    strafe_right: bool,
    mouse_dx: f64,
}

// ── Raycasting ────────────────────────────────────────────────────

struct HitResult {
    dist: f64,
    wall: u8,
    side: bool,  // false = NS wall, true = EW wall
    tex_x: f64,  // 0..1 where ray hit on wall face
}

fn cast_ray(px: f64, py: f64, angle: f64) -> HitResult {
    let dir_x = angle.cos();
    let dir_y = angle.sin();

    let mut map_x = px as i32;
    let mut map_y = py as i32;

    let delta_x = if dir_x == 0.0 { 1e30 } else { (1.0 / dir_x).abs() };
    let delta_y = if dir_y == 0.0 { 1e30 } else { (1.0 / dir_y).abs() };

    let (step_x, mut side_x) = if dir_x < 0.0 {
        (-1i32, (px - map_x as f64) * delta_x)
    } else {
        (1i32, (map_x as f64 + 1.0 - px) * delta_x)
    };
    let (step_y, mut side_y) = if dir_y < 0.0 {
        (-1i32, (py - map_y as f64) * delta_y)
    } else {
        (1i32, (map_y as f64 + 1.0 - py) * delta_y)
    };

    let mut side = false;
    loop {
        if side_x < side_y {
            side_x += delta_x;
            map_x += step_x;
            side = false;
        } else {
            side_y += delta_y;
            map_y += step_y;
            side = true;
        }

        let wall = map_at(map_x as usize, map_y as usize);
        if wall > 0 {
            let dist = if !side {
                (map_x as f64 - px + (1.0 - step_x as f64) / 2.0) / dir_x
            } else {
                (map_y as f64 - py + (1.0 - step_y as f64) / 2.0) / dir_y
            };

            let tex_x = if !side {
                let hit = py + dist * dir_y;
                hit - hit.floor()
            } else {
                let hit = px + dist * dir_x;
                hit - hit.floor()
            };

            return HitResult { dist: dist.max(0.001), wall, side, tex_x };
        }

        if side_x > MAX_DIST && side_y > MAX_DIST {
            return HitResult { dist: MAX_DIST, wall: 0, side: false, tex_x: 0.0 };
        }
    }
}

// ── Rendering ─────────────────────────────────────────────────────

fn render_frame(pixels: &mut [u8], player: &Player) {
    let half_h = H as f64 / 2.0;

    for x in 0..W {
        let ray_angle = player.angle - FOV / 2.0 + (x as f64 / W as f64) * FOV;
        let hit = cast_ray(player.x, player.y, ray_angle);

        // Fix fisheye
        let perp_dist = hit.dist * (ray_angle - player.angle).cos();

        // Wall height
        let wall_h = (H as f64 / perp_dist).min(H as f64 * 4.0);
        let wall_top = ((half_h - wall_h / 2.0) as usize).min(H);
        let wall_bot = ((half_h + wall_h / 2.0) as usize).min(H);

        let (wr, wg, wb) = if hit.wall > 0 {
            wall_color(hit.wall, hit.side)
        } else {
            (30, 30, 40)
        };

        for y in 0..H {
            let idx = (y * W + x) * 4;

            if y < wall_top {
                // Ceiling — gradient
                let f = 1.0 - (y as f64 / half_h);
                let ci = (30.0 * f) as u8;
                pixels[idx]     = ci / 3;
                pixels[idx + 1] = ci / 3;
                pixels[idx + 2] = ci;
                pixels[idx + 3] = 255;
            } else if y < wall_bot {
                // Wall — add vertical stripe texture
                let stripe = ((hit.tex_x * 8.0) as u32 % 2 == 0) as u8;
                let shade = 1.0 - (perp_dist / MAX_DIST).min(1.0) * 0.7;
                let s = stripe as f64 * 0.05 + 0.95;
                pixels[idx]     = ((wr as f64) * shade * s) as u8;
                pixels[idx + 1] = ((wg as f64) * shade * s) as u8;
                pixels[idx + 2] = ((wb as f64) * shade * s) as u8;
                pixels[idx + 3] = 255;
            } else {
                // Floor — gradient
                let f = (y as f64 - half_h) / half_h;
                let fi = (40.0 * f) as u8;
                pixels[idx]     = fi / 3;
                pixels[idx + 1] = fi / 2;
                pixels[idx + 2] = fi / 3;
                pixels[idx + 3] = 255;
            }
        }
    }
}

fn render_minimap(pixels: &mut [u8], player: &Player) {
    let scale = 4;
    let ox = W - MAP_W * scale - 8;
    let oy = 8;

    for my in 0..MAP_H {
        for mx in 0..MAP_W {
            let wall = map_at(mx, my);
            let (r, g, b) = if wall > 0 {
                let (wr, wg, wb) = wall_color(wall, false);
                (wr, wg, wb)
            } else {
                (20, 22, 30)
            };
            for dy in 0..scale {
                for dx in 0..scale {
                    let px = ox + mx * scale + dx;
                    let py = oy + my * scale + dy;
                    if px < W && py < H {
                        let idx = (py * W + px) * 4;
                        pixels[idx] = r; pixels[idx+1] = g; pixels[idx+2] = b; pixels[idx+3] = 200;
                    }
                }
            }
        }
    }

    // Player dot
    let ppx = ox + (player.x * scale as f64) as usize;
    let ppy = oy + (player.y * scale as f64) as usize;
    for dy in 0..3usize {
        for dx in 0..3usize {
            let px = ppx + dx;
            let py = ppy + dy;
            if px < W && py < H {
                let idx = (py * W + px) * 4;
                pixels[idx] = 255; pixels[idx+1] = 80; pixels[idx+2] = 80; pixels[idx+3] = 255;
            }
        }
    }

    // Direction line
    let lx = (player.angle.cos() * 8.0) as i32;
    let ly = (player.angle.sin() * 8.0) as i32;
    for t in 0..8 {
        let px = (ppx as i32 + lx * t / 8) as usize;
        let py = (ppy as i32 + ly * t / 8) as usize;
        if px < W && py < H {
            let idx = (py * W + px) * 4;
            pixels[idx] = 255; pixels[idx+1] = 200; pixels[idx+2] = 80; pixels[idx+3] = 255;
        }
    }
}

// ── Entry Point ───────────────────────────────────────────────────

pub fn run() {
    console_error_panic_hook::set_once();

    let doc = document();
    let root = doc.get_element_by_id("app").unwrap();

    let container = el("div", "");
    attr(&container, "style",
        "display:flex;flex-direction:column;align-items:center;padding:1rem");

    let title = text_el("h1", "ox-h3 ox-font-display ox-text-center ox-mb-2", "ox∅ Doom");
    let sub = text_el("p", "ox-text-center ox-mb-3",
        "WASD to move, mouse to look. Click canvas to capture pointer.");
    attr(&sub, "style", "color:#9ca0ab;font-size:0.875rem");

    let canvas = doc.create_element("canvas").unwrap();
    canvas.set_attribute("width", &W.to_string()).unwrap();
    canvas.set_attribute("height", &H.to_string()).unwrap();
    canvas.set_attribute("style", &format!(
        "width:{}px;height:{}px;display:block;border-radius:0.75rem;\
         border:1px solid #2e3140;cursor:crosshair;image-rendering:pixelated",
        W, H
    )).unwrap();

    append(&container, &[&title, &sub, &canvas]);
    root.append_child(&container).unwrap();

    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
    let ctx = canvas
        .get_context("2d").unwrap().unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();

    let player = Rc::new(RefCell::new(Player { x: 8.0, y: 8.0, angle: 0.0 }));
    let input = Rc::new(RefCell::new(Input {
        forward: false, backward: false, left: false, right: false,
        strafe_left: false, strafe_right: false, mouse_dx: 0.0,
    }));

    // Pointer lock on click
    {
        let canvas_ref = canvas.clone();
        let cb = Closure::wrap(Box::new(move |_: web_sys::Event| {
            canvas_ref.request_pointer_lock();
        }) as Box<dyn Fn(web_sys::Event)>);
        canvas.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }

    // Mouse move (pointer locked)
    {
        let input = input.clone();
        let cb = Closure::wrap(Box::new(move |e: web_sys::Event| {
            let me: &web_sys::MouseEvent = e.unchecked_ref();
            input.borrow_mut().mouse_dx += me.movement_x() as f64;
        }) as Box<dyn Fn(web_sys::Event)>);
        doc.add_event_listener_with_callback("mousemove", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }

    // Keyboard
    {
        let input = input.clone();
        let cb = Closure::wrap(Box::new(move |e: web_sys::Event| {
            let ke: &web_sys::KeyboardEvent = e.unchecked_ref();
            let pressed = e.type_() == "keydown";
            let mut inp = input.borrow_mut();
            match ke.key().as_str() {
                "w" | "W" | "ArrowUp"    => inp.forward = pressed,
                "s" | "S" | "ArrowDown"  => inp.backward = pressed,
                "a" | "A"               => inp.strafe_left = pressed,
                "d" | "D"               => inp.strafe_right = pressed,
                "ArrowLeft"             => inp.left = pressed,
                "ArrowRight"            => inp.right = pressed,
                _ => {}
            }
            e.prevent_default();
        }) as Box<dyn Fn(web_sys::Event)>);
        doc.add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref()).unwrap();
        doc.add_event_listener_with_callback("keyup", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }

    // Pixel buffer
    let mut pixels = vec![0u8; W * H * 4];

    // Animation loop
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        // Update player
        {
            let inp = input.borrow();
            let mut p = player.borrow_mut();

            // Mouse look
            p.angle += inp.mouse_dx * 0.003;

            let cos_a = p.angle.cos();
            let sin_a = p.angle.sin();

            let mut dx = 0.0f64;
            let mut dy = 0.0f64;

            if inp.forward  { dx += cos_a * MOVE_SPEED; dy += sin_a * MOVE_SPEED; }
            if inp.backward { dx -= cos_a * MOVE_SPEED; dy -= sin_a * MOVE_SPEED; }
            if inp.strafe_left  { dx += sin_a * MOVE_SPEED; dy -= cos_a * MOVE_SPEED; }
            if inp.strafe_right { dx -= sin_a * MOVE_SPEED; dy += cos_a * MOVE_SPEED; }
            if inp.left  { p.angle -= ROT_SPEED; }
            if inp.right { p.angle += ROT_SPEED; }

            // Collision detection (slide along walls)
            let margin = 0.2;
            if map_at((p.x + dx + margin * dx.signum()) as usize, p.y as usize) == 0 {
                p.x += dx;
            }
            if map_at(p.x as usize, (p.y + dy + margin * dy.signum()) as usize) == 0 {
                p.y += dy;
            }
        }
        input.borrow_mut().mouse_dx = 0.0;

        // Render
        {
            let p = player.borrow();
            render_frame(&mut pixels, &p);
            render_minimap(&mut pixels, &p);
        }

        // Draw to canvas
        let data = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(&pixels), W as u32, H as u32
        ).unwrap();
        ctx.put_image_data(&data, 0.0, 0.0).unwrap();

        // Crosshair
        ctx.set_stroke_style_str("rgba(255,255,255,0.4)");
        ctx.set_line_width(1.0);
        let cx = W as f64 / 2.0;
        let cy = H as f64 / 2.0;
        ctx.begin_path();
        ctx.move_to(cx - 10.0, cy); ctx.line_to(cx + 10.0, cy);
        ctx.move_to(cx, cy - 10.0); ctx.line_to(cx, cy + 10.0);
        ctx.stroke();

        web_sys::window().unwrap()
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }) as Box<dyn FnMut()>));

    web_sys::window().unwrap()
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();
}
