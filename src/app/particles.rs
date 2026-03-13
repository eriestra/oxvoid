//! ox∅ Particle Life — emergent behavior from simple attraction/repulsion rules.
//! Thousands of colored particles self-organize into digital organisms.

use crate::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::cell::RefCell;
use std::rc::Rc;

const NUM_COLORS: usize = 6;
const PARTICLES_PER_COLOR: usize = 200;
const NUM_PARTICLES: usize = NUM_COLORS * PARTICLES_PER_COLOR;
const CANVAS_SIZE: f64 = 600.0;
const FRICTION: f32 = 0.5;
const MAX_RADIUS: f32 = 120.0;
const MIN_RADIUS: f32 = 20.0;
const FORCE_SCALE: f32 = 0.8;
const DT: f32 = 0.02;

// ── Particle ──────────────────────────────────────────────────────

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    color: usize,
}

// ── Simulation ────────────────────────────────────────────────────

struct Sim {
    particles: Vec<Particle>,
    rules: [[f32; NUM_COLORS]; NUM_COLORS], // attraction matrix
    size: f32,
}

impl Sim {
    fn new() -> Self {
        let size = CANVAS_SIZE as f32;
        let mut particles = Vec::with_capacity(NUM_PARTICLES);

        // Simple PRNG (xorshift32)
        let mut seed: u32 = 42;
        let mut rand = || -> f32 {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            (seed as f32) / (u32::MAX as f32)
        };

        for color in 0..NUM_COLORS {
            for _ in 0..PARTICLES_PER_COLOR {
                particles.push(Particle {
                    x: rand() * size,
                    y: rand() * size,
                    vx: 0.0,
                    vy: 0.0,
                    color,
                });
            }
        }

        // Random attraction rules: -1 (repel) to +1 (attract)
        let mut rules = [[0.0f32; NUM_COLORS]; NUM_COLORS];
        for i in 0..NUM_COLORS {
            for j in 0..NUM_COLORS {
                rules[i][j] = rand() * 2.0 - 1.0;
            }
        }

        Self { particles, rules, size }
    }

    fn randomize_rules(&mut self) {
        let mut seed: u32 = (js_sys::Date::now() as u32).wrapping_mul(2654435761);
        let mut rand = || -> f32 {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            (seed as f32) / (u32::MAX as f32)
        };
        for i in 0..NUM_COLORS {
            for j in 0..NUM_COLORS {
                self.rules[i][j] = rand() * 2.0 - 1.0;
            }
        }
    }

    fn step(&mut self) {
        let n = self.particles.len();

        // Compute forces (O(n²) but WASM makes it fast)
        // Collect positions and colors first to avoid borrow issues
        let positions: Vec<(f32, f32, usize)> = self.particles.iter()
            .map(|p| (p.x, p.y, p.color))
            .collect();

        for i in 0..n {
            let (ax, ay, ci) = positions[i];
            let mut fx = 0.0f32;
            let mut fy = 0.0f32;

            for j in 0..n {
                if i == j { continue; }
                let (bx, by, cj) = positions[j];

                let mut dx = bx - ax;
                let mut dy = by - ay;

                // Wrap around (toroidal space)
                if dx > self.size * 0.5 { dx -= self.size; }
                if dx < -self.size * 0.5 { dx += self.size; }
                if dy > self.size * 0.5 { dy -= self.size; }
                if dy < -self.size * 0.5 { dy += self.size; }

                let dist = (dx * dx + dy * dy).sqrt();
                if dist > MAX_RADIUS || dist < 0.01 { continue; }

                let rule = self.rules[ci][cj];

                // Force curve: repel at close range, attract/repel at medium range
                let force = if dist < MIN_RADIUS {
                    // Strong repulsion at very close range (prevents overlap)
                    (dist / MIN_RADIUS - 1.0) * FORCE_SCALE
                } else {
                    // Attraction/repulsion based on rule
                    rule * FORCE_SCALE * (1.0 - (2.0 * (dist - MIN_RADIUS) / (MAX_RADIUS - MIN_RADIUS) - 1.0).abs())
                };

                fx += force * dx / dist;
                fy += force * dy / dist;
            }

            let p = &mut self.particles[i];
            p.vx = (p.vx + fx * DT) * (1.0 - FRICTION * DT);
            p.vy = (p.vy + fy * DT) * (1.0 - FRICTION * DT);
            p.x += p.vx;
            p.y += p.vy;

            // Wrap around
            if p.x < 0.0 { p.x += self.size; }
            if p.x >= self.size { p.x -= self.size; }
            if p.y < 0.0 { p.y += self.size; }
            if p.y >= self.size { p.y -= self.size; }
        }
    }
}

// ── Colors ────────────────────────────────────────────────────────

const PALETTE: [(f64, f64, f64); NUM_COLORS] = [
    (1.0, 0.3, 0.3),   // red
    (0.3, 1.0, 0.5),   // green
    (0.3, 0.5, 1.0),   // blue
    (1.0, 0.9, 0.2),   // yellow
    (0.8, 0.3, 1.0),   // purple
    (0.2, 0.9, 0.9),   // cyan
];

// ── Entry Point ───────────────────────────────────────────────────


pub fn run() {
    console_error_panic_hook::set_once();

    let doc = document();
    let root = doc.get_element_by_id("app").unwrap();

    // Container
    let container = el("div", "");
    attr(&container, "style",
        "display:flex;flex-direction:column;align-items:center;padding:2rem;font-family:var(--ox-font-sans)");

    let title = text_el("h1", "ox-h3 ox-font-display ox-text-center ox-mb-2", "Particle Life");
    let sub = text_el("p", "ox-text-center ox-mb-4", "Click and drag to disturb. Press R to randomize rules.");
    attr(&sub, "style", "color:#9ca0ab;font-size:0.875rem");

    // Canvas
    let canvas = doc.create_element("canvas").unwrap();
    let cs = CANVAS_SIZE as u32;
    canvas.set_attribute("width", &cs.to_string()).unwrap();
    canvas.set_attribute("height", &cs.to_string()).unwrap();
    canvas.set_attribute("style",
        "width:100%;max-width:600px;aspect-ratio:1;display:block;cursor:crosshair;\
         border-radius:0.75rem;background:#0a0c10;border:1px solid #2e3140"
    ).unwrap();

    // Randomize button
    let btn = text_el("button", "ox-btn ox-btn-outline ox-btn-sm ox-mt-4", "Randomize Rules (R)");

    append(&container, &[&title, &sub, &canvas, &btn]);
    root.append_child(&container).unwrap();

    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
    let ctx = canvas
        .get_context("2d").unwrap().unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();

    // Shared state
    let sim = Rc::new(RefCell::new(Sim::new()));
    let mouse = Rc::new(RefCell::new((0.0f32, 0.0f32, false)));

    // Mouse events
    {
        let mouse = mouse.clone();
        let canvas_ref = canvas.clone();
        let cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            let rect = canvas_ref.get_bounding_client_rect();
            let scale = CANVAS_SIZE / rect.width();
            let x = ((e.client_x() as f64 - rect.left()) * scale) as f32;
            let y = ((e.client_y() as f64 - rect.top()) * scale) as f32;
            let mut m = mouse.borrow_mut();
            m.0 = x; m.1 = y;
        }) as Box<dyn Fn(web_sys::MouseEvent)>);
        canvas.add_event_listener_with_callback("mousemove", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }
    {
        let mouse = mouse.clone();
        let cb = Closure::wrap(Box::new(move |_: web_sys::MouseEvent| {
            mouse.borrow_mut().2 = true;
        }) as Box<dyn Fn(web_sys::MouseEvent)>);
        canvas.add_event_listener_with_callback("mousedown", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }
    {
        let mouse = mouse.clone();
        let cb = Closure::wrap(Box::new(move |_: web_sys::MouseEvent| {
            mouse.borrow_mut().2 = false;
        }) as Box<dyn Fn(web_sys::MouseEvent)>);
        canvas.add_event_listener_with_callback("mouseup", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }

    // Keyboard: R to randomize
    {
        let sim = sim.clone();
        let cb = Closure::wrap(Box::new(move |e: web_sys::Event| {
            let ke: &web_sys::KeyboardEvent = e.unchecked_ref();
            if ke.key() == "r" || ke.key() == "R" {
                sim.borrow_mut().randomize_rules();
            }
        }) as Box<dyn Fn(web_sys::Event)>);
        doc.add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }

    // Button randomize
    {
        let sim = sim.clone();
        let cb = Closure::wrap(Box::new(move |_: web_sys::Event| {
            sim.borrow_mut().randomize_rules();
        }) as Box<dyn Fn(web_sys::Event)>);
        btn.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }

    // Animation loop
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        // Mouse push force
        {
            let m = mouse.borrow();
            if m.2 {
                let mut sim = sim.borrow_mut();
                for p in sim.particles.iter_mut() {
                    let dx = p.x - m.0;
                    let dy = p.y - m.1;
                    let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                    if dist < 80.0 {
                        let force = (1.0 - dist / 80.0) * 3.0;
                        p.vx += dx / dist * force;
                        p.vy += dy / dist * force;
                    }
                }
            }
        }

        // Step
        sim.borrow_mut().step();

        // Render
        ctx.set_global_alpha(0.15);
        ctx.set_fill_style_str("black");
        ctx.fill_rect(0.0, 0.0, CANVAS_SIZE, CANVAS_SIZE);
        ctx.set_global_alpha(0.85);

        let sim = sim.borrow();
        for p in &sim.particles {
            let (r, g, b) = PALETTE[p.color];
            ctx.set_fill_style_str(&format!("rgb({},{},{})", (r*255.0) as u8, (g*255.0) as u8, (b*255.0) as u8));
            ctx.begin_path();
            ctx.arc(p.x as f64, p.y as f64, 2.0, 0.0, std::f64::consts::TAU).unwrap();
            ctx.fill();
        }

        // Request next frame
        web_sys::window().unwrap()
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }) as Box<dyn FnMut()>));

    web_sys::window().unwrap()
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();
}
