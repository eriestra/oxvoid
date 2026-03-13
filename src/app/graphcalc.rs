//! ox∅ Graphing Calculator — plot math functions in real-time.
//! Type expressions like sin(x), x^2, cos(x)*x, etc.

use crate::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::cell::RefCell;
use std::rc::Rc;

const CANVAS_W: f64 = 700.0;
const CANVAS_H: f64 = 500.0;

// ── Expression Parser ─────────────────────────────────────────────

#[derive(Clone, Debug)]
enum Expr {
    Num(f64),
    Var,  // x
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
    Fn1(String, Box<Expr>),  // sin, cos, tan, abs, sqrt, log, ln, exp
}

impl Expr {
    fn eval(&self, x: f64) -> f64 {
        match self {
            Expr::Num(n) => *n,
            Expr::Var => x,
            Expr::Add(a, b) => a.eval(x) + b.eval(x),
            Expr::Sub(a, b) => a.eval(x) - b.eval(x),
            Expr::Mul(a, b) => a.eval(x) * b.eval(x),
            Expr::Div(a, b) => { let d = b.eval(x); if d.abs() < 1e-12 { f64::NAN } else { a.eval(x) / d } },
            Expr::Pow(a, b) => a.eval(x).powf(b.eval(x)),
            Expr::Neg(a) => -a.eval(x),
            Expr::Fn1(name, a) => {
                let v = a.eval(x);
                match name.as_str() {
                    "sin" => v.sin(), "cos" => v.cos(), "tan" => v.tan(),
                    "abs" => v.abs(), "sqrt" => v.sqrt(), "log" | "log10" => v.log10(),
                    "ln" => v.ln(), "exp" => v.exp(), "asin" => v.asin(),
                    "acos" => v.acos(), "atan" => v.atan(), "floor" => v.floor(),
                    "ceil" => v.ceil(), "round" => v.round(),
                    _ => f64::NAN,
                }
            }
        }
    }
}

// Recursive descent parser
struct Parser { chars: Vec<char>, pos: usize }

impl Parser {
    fn new(input: &str) -> Self {
        Self { chars: input.chars().collect(), pos: 0 }
    }

    fn peek(&self) -> Option<char> { self.chars.get(self.pos).copied() }
    fn next(&mut self) -> Option<char> { let c = self.peek(); self.pos += 1; c }
    fn skip_ws(&mut self) { while self.peek().map_or(false, |c| c.is_whitespace()) { self.pos += 1; } }

    fn parse(&mut self) -> Option<Expr> {
        let e = self.parse_add()?;
        Some(e)
    }

    fn parse_add(&mut self) -> Option<Expr> {
        let mut left = self.parse_mul()?;
        loop {
            self.skip_ws();
            match self.peek() {
                Some('+') => { self.next(); left = Expr::Add(Box::new(left), Box::new(self.parse_mul()?)); }
                Some('-') => { self.next(); left = Expr::Sub(Box::new(left), Box::new(self.parse_mul()?)); }
                _ => break,
            }
        }
        Some(left)
    }

    fn parse_mul(&mut self) -> Option<Expr> {
        let mut left = self.parse_pow()?;
        loop {
            self.skip_ws();
            match self.peek() {
                Some('*') => { self.next(); left = Expr::Mul(Box::new(left), Box::new(self.parse_pow()?)); }
                Some('/') => { self.next(); left = Expr::Div(Box::new(left), Box::new(self.parse_pow()?)); }
                _ => break,
            }
        }
        Some(left)
    }

    fn parse_pow(&mut self) -> Option<Expr> {
        let base = self.parse_unary()?;
        self.skip_ws();
        if self.peek() == Some('^') {
            self.next();
            let exp = self.parse_unary()?;
            Some(Expr::Pow(Box::new(base), Box::new(exp)))
        } else {
            Some(base)
        }
    }

    fn parse_unary(&mut self) -> Option<Expr> {
        self.skip_ws();
        if self.peek() == Some('-') {
            self.next();
            Some(Expr::Neg(Box::new(self.parse_atom()?)))
        } else {
            self.parse_atom()
        }
    }

    fn parse_atom(&mut self) -> Option<Expr> {
        self.skip_ws();
        match self.peek()? {
            '(' => {
                self.next(); // (
                let e = self.parse_add()?;
                self.skip_ws();
                if self.peek() == Some(')') { self.next(); }
                Some(e)
            }
            'x' | 'X' => { self.next(); Some(Expr::Var) }
            'p' | 'P' => {
                // pi
                let start = self.pos;
                self.next();
                if self.peek() == Some('i') || self.peek() == Some('I') {
                    self.next();
                    Some(Expr::Num(std::f64::consts::PI))
                } else {
                    self.pos = start;
                    None
                }
            }
            'e' if self.chars.get(self.pos + 1).map_or(true, |c| !c.is_alphabetic()) => {
                self.next();
                Some(Expr::Num(std::f64::consts::E))
            }
            c if c.is_alphabetic() => {
                // Function name
                let start = self.pos;
                while self.peek().map_or(false, |c| c.is_alphabetic()) { self.pos += 1; }
                let name: String = self.chars[start..self.pos].iter().collect();
                self.skip_ws();
                if self.peek() == Some('(') {
                    self.next();
                    let arg = self.parse_add()?;
                    self.skip_ws();
                    if self.peek() == Some(')') { self.next(); }
                    // Handle implicit multiplication: sin(x)x -> sin(x)*x
                    let mut result = Expr::Fn1(name.to_lowercase(), Box::new(arg));
                    self.skip_ws();
                    if self.peek().map_or(false, |c| c == 'x' || c == '(' || c.is_alphabetic()) {
                        let right = self.parse_atom()?;
                        result = Expr::Mul(Box::new(result), Box::new(right));
                    }
                    Some(result)
                } else {
                    // Maybe it's a constant
                    match name.to_lowercase().as_str() {
                        "pi" => Some(Expr::Num(std::f64::consts::PI)),
                        "e" => Some(Expr::Num(std::f64::consts::E)),
                        _ => None,
                    }
                }
            }
            c if c.is_ascii_digit() || c == '.' => {
                let start = self.pos;
                while self.peek().map_or(false, |c| c.is_ascii_digit() || c == '.') { self.pos += 1; }
                let s: String = self.chars[start..self.pos].iter().collect();
                let n = s.parse::<f64>().ok()?;
                // Implicit multiplication: 2x -> 2*x, 2( -> 2*(
                self.skip_ws();
                if self.peek().map_or(false, |c| c == 'x' || c == '(' || c.is_alphabetic()) {
                    let right = self.parse_atom()?;
                    Some(Expr::Mul(Box::new(Expr::Num(n)), Box::new(right)))
                } else {
                    Some(Expr::Num(n))
                }
            }
            _ => None,
        }
    }
}

fn parse_expr(input: &str) -> Option<Expr> {
    let mut p = Parser::new(input);
    p.parse()
}

// ── Graph State ───────────────────────────────────────────────────

struct GraphState {
    center_x: f64,
    center_y: f64,
    zoom: f64, // pixels per unit
    dragging: bool,
    drag_start_x: f64,
    drag_start_y: f64,
    drag_center_x: f64,
    drag_center_y: f64,
}

impl GraphState {
    fn new() -> Self {
        Self {
            center_x: 0.0, center_y: 0.0, zoom: 50.0,
            dragging: false,
            drag_start_x: 0.0, drag_start_y: 0.0,
            drag_center_x: 0.0, drag_center_y: 0.0,
        }
    }

    fn screen_to_math_x(&self, sx: f64) -> f64 { (sx - CANVAS_W / 2.0) / self.zoom + self.center_x }
    fn screen_to_math_y(&self, sy: f64) -> f64 { -(sy - CANVAS_H / 2.0) / self.zoom + self.center_y }
    fn math_to_screen_x(&self, mx: f64) -> f64 { (mx - self.center_x) * self.zoom + CANVAS_W / 2.0 }
    fn math_to_screen_y(&self, my: f64) -> f64 { -(my - self.center_y) * self.zoom + CANVAS_H / 2.0 }
}

// ── Colors for multiple functions ─────────────────────────────────

const PLOT_COLORS: [&str; 6] = [
    "#5c7cfa", // blue
    "#fa5252", // red
    "#40c057", // green
    "#fab005", // yellow
    "#be4bdb", // purple
    "#20c997", // teal
];

// ── Rendering ─────────────────────────────────────────────────────

fn draw_graph(
    ctx: &web_sys::CanvasRenderingContext2d,
    graph: &GraphState,
    exprs: &[(String, Option<Expr>)],
    mouse_x: f64,
) {
    // Background
    ctx.set_fill_style_str("#0f1117");
    ctx.fill_rect(0.0, 0.0, CANVAS_W, CANVAS_H);

    // Grid
    draw_grid(ctx, graph);

    // Plot each function
    for (i, (_, expr)) in exprs.iter().enumerate() {
        if let Some(expr) = expr {
            draw_function(ctx, graph, expr, PLOT_COLORS[i % PLOT_COLORS.len()]);
        }
    }

    // Crosshair at mouse position
    if mouse_x >= 0.0 && mouse_x <= CANVAS_W {
        let mx = graph.screen_to_math_x(mouse_x);
        ctx.set_stroke_style_str("rgba(255,255,255,0.15)");
        ctx.set_line_width(1.0);
        ctx.begin_path();
        ctx.move_to(mouse_x, 0.0);
        ctx.line_to(mouse_x, CANVAS_H);
        ctx.stroke();

        // Show values at cursor
        ctx.set_font("11px 'JetBrains Mono', monospace");
        ctx.set_text_align("left");
        let mut label_y = 20.0;
        for (i, (text, expr)) in exprs.iter().enumerate() {
            if let Some(expr) = expr {
                let y = expr.eval(mx);
                if y.is_finite() {
                    let sy = graph.math_to_screen_y(y);
                    // Dot on curve
                    ctx.set_fill_style_str(PLOT_COLORS[i % PLOT_COLORS.len()]);
                    ctx.begin_path();
                    ctx.arc(mouse_x, sy, 4.0, 0.0, std::f64::consts::TAU).unwrap();
                    ctx.fill();
                    // Label
                    ctx.fill_text(&format!("f{}({:.2}) = {:.4}", i+1, mx, y), 10.0, label_y).ok();
                    label_y += 16.0;
                }
            }
        }

        // X coordinate
        ctx.set_fill_style_str("rgba(255,255,255,0.4)");
        ctx.fill_text(&format!("x = {:.3}", mx), 10.0, CANVAS_H - 10.0).ok();
    }
}

fn draw_grid(ctx: &web_sys::CanvasRenderingContext2d, g: &GraphState) {
    // Compute grid spacing that looks good at any zoom
    let target_px = 60.0; // target pixel spacing between grid lines
    let raw = target_px / g.zoom;
    let mag = 10.0f64.powf(raw.log10().floor());
    let residual = raw / mag;
    let step = if residual < 1.5 { mag } else if residual < 3.5 { 2.0 * mag } else if residual < 7.5 { 5.0 * mag } else { 10.0 * mag };

    let x_min = g.screen_to_math_x(0.0);
    let x_max = g.screen_to_math_x(CANVAS_W);
    let y_min = g.screen_to_math_y(CANVAS_H);
    let y_max = g.screen_to_math_y(0.0);

    // Grid lines
    ctx.set_stroke_style_str("rgba(255,255,255,0.06)");
    ctx.set_line_width(1.0);

    let mut gx = (x_min / step).floor() * step;
    while gx <= x_max {
        let sx = g.math_to_screen_x(gx);
        ctx.begin_path(); ctx.move_to(sx, 0.0); ctx.line_to(sx, CANVAS_H); ctx.stroke();
        gx += step;
    }
    let mut gy = (y_min / step).floor() * step;
    while gy <= y_max {
        let sy = g.math_to_screen_y(gy);
        ctx.begin_path(); ctx.move_to(0.0, sy); ctx.line_to(CANVAS_W, sy); ctx.stroke();
        gy += step;
    }

    // Axes
    ctx.set_stroke_style_str("rgba(255,255,255,0.25)");
    ctx.set_line_width(1.5);
    let ox = g.math_to_screen_x(0.0);
    let oy = g.math_to_screen_y(0.0);
    if ox >= 0.0 && ox <= CANVAS_W {
        ctx.begin_path(); ctx.move_to(ox, 0.0); ctx.line_to(ox, CANVAS_H); ctx.stroke();
    }
    if oy >= 0.0 && oy <= CANVAS_H {
        ctx.begin_path(); ctx.move_to(0.0, oy); ctx.line_to(CANVAS_W, oy); ctx.stroke();
    }

    // Tick labels
    ctx.set_fill_style_str("rgba(255,255,255,0.3)");
    ctx.set_font("10px 'JetBrains Mono', monospace");
    ctx.set_text_align("center");

    let mut gx = (x_min / step).floor() * step;
    while gx <= x_max {
        if gx.abs() > step * 0.1 {
            let sx = g.math_to_screen_x(gx);
            let label = if step >= 1.0 { format!("{:.0}", gx) } else { format!("{:.1}", gx) };
            ctx.fill_text(&label, sx, oy.clamp(12.0, CANVAS_H - 4.0) + 12.0).ok();
        }
        gx += step;
    }
    ctx.set_text_align("right");
    let mut gy = (y_min / step).floor() * step;
    while gy <= y_max {
        if gy.abs() > step * 0.1 {
            let sy = g.math_to_screen_y(gy);
            let label = if step >= 1.0 { format!("{:.0}", gy) } else { format!("{:.1}", gy) };
            ctx.fill_text(&label, ox.clamp(4.0, CANVAS_W - 4.0) - 4.0, sy + 3.0).ok();
        }
        gy += step;
    }
}

fn draw_function(ctx: &web_sys::CanvasRenderingContext2d, g: &GraphState, expr: &Expr, color: &str) {
    ctx.set_stroke_style_str(color);
    ctx.set_line_width(2.0);
    ctx.begin_path();

    let mut started = false;
    let step = 1.0; // 1 pixel step
    let mut prev_y = f64::NAN;

    for px in 0..(CANVAS_W as usize) {
        let mx = g.screen_to_math_x(px as f64);
        let my = expr.eval(mx);

        if !my.is_finite() || my.abs() > 1e6 {
            started = false;
            prev_y = f64::NAN;
            continue;
        }

        let sy = g.math_to_screen_y(my);

        // Detect discontinuities (large jumps)
        if prev_y.is_finite() && (sy - prev_y).abs() > CANVAS_H * 0.8 {
            started = false;
        }

        if !started {
            ctx.move_to(px as f64, sy);
            started = true;
        } else {
            ctx.line_to(px as f64, sy);
        }
        prev_y = sy;
    }
    ctx.stroke();
}

// ── Entry Point ───────────────────────────────────────────────────

pub fn run() {
    console_error_panic_hook::set_once();

    let doc = document();
    let root = doc.get_element_by_id("app").unwrap();

    let container = el("div", "");
    attr(&container, "style",
        "display:flex;flex-direction:column;align-items:center;padding:1.5rem;max-width:740px;margin:0 auto");

    let title = text_el("h1", "ox-h3 ox-font-display ox-text-center ox-mb-3", "Graphing Calculator");

    // Function inputs
    let inputs_container = el("div", "ox-stack ox-gap-2 ox-mb-3");
    attr(&inputs_container, "style", "width:100%");

    let default_exprs = ["sin(x)", "x^2/10", "cos(x)*2"];

    let exprs: Rc<RefCell<Vec<(String, Option<Expr>)>>> = Rc::new(RefCell::new(Vec::new()));

    let input_els: Vec<web_sys::Element> = (0..3).map(|i| {
        let row = el("div", "ox-flex ox-gap-2 ox-items-center");
        let color_dot = el("span", "");
        attr(&color_dot, "style", &format!(
            "width:12px;height:12px;border-radius:50%;background:{};flex-shrink:0",
            PLOT_COLORS[i]
        ));
        let label = text_el("span", "ox-text-sm ox-font-mono", &format!("f{}", i + 1));
        attr(&label, "style", "color:#9ca0ab;width:20px;flex-shrink:0");
        let input = el("input", "ox-input ox-input-sm");
        attr(&input, "type", "text");
        attr(&input, "placeholder", "e.g. sin(x), x^2, cos(2x)+1");
        if i < default_exprs.len() {
            attr(&input, "value", default_exprs[i]);
        }
        append(&row, &[&color_dot, &label, &input]);
        inputs_container.append_child(&row).unwrap();

        // Parse default
        let text = if i < default_exprs.len() { default_exprs[i].to_string() } else { String::new() };
        let parsed = parse_expr(&text);
        exprs.borrow_mut().push((text, parsed));

        input.clone()
    }).collect();

    // Wire input events
    for (i, input_el) in input_els.iter().enumerate() {
        let exprs = exprs.clone();
        let cb = Closure::wrap(Box::new(move |e: web_sys::Event| {
            let target = e.target().unwrap();
            let input: &web_sys::HtmlInputElement = target.unchecked_ref();
            let text = input.value();
            let parsed = parse_expr(&text);
            let mut ex = exprs.borrow_mut();
            ex[i] = (text, parsed);
        }) as Box<dyn Fn(web_sys::Event)>);
        input_el.add_event_listener_with_callback("input", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }

    let help = text_el("p", "ox-text-xs ox-text-center", "Supports: sin cos tan abs sqrt log ln exp asin acos atan floor ceil | pi e | x^n | 2x (implicit multiply) | scroll to zoom, drag to pan");
    attr(&help, "style", "color:#5c6170;margin-bottom:0.75rem");

    // Canvas
    let canvas = doc.create_element("canvas").unwrap();
    canvas.set_attribute("width", &(CANVAS_W as u32).to_string()).unwrap();
    canvas.set_attribute("height", &(CANVAS_H as u32).to_string()).unwrap();
    canvas.set_attribute("style",
        "width:100%;max-width:700px;aspect-ratio:7/5;display:block;\
         border-radius:0.75rem;border:1px solid #2e3140;cursor:crosshair"
    ).unwrap();

    append(&container, &[&title, &inputs_container, &help, &canvas]);
    root.append_child(&container).unwrap();

    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
    let ctx = canvas.get_context("2d").unwrap().unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();

    let graph = Rc::new(RefCell::new(GraphState::new()));
    let mouse_x = Rc::new(RefCell::new(-1.0f64));

    // Mouse move
    {
        let mouse_x = mouse_x.clone();
        let canvas_ref = canvas.clone();
        let cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            let rect = canvas_ref.get_bounding_client_rect();
            let scale = CANVAS_W / rect.width();
            *mouse_x.borrow_mut() = (e.client_x() as f64 - rect.left()) * scale;
        }) as Box<dyn Fn(web_sys::MouseEvent)>);
        canvas.add_event_listener_with_callback("mousemove", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }

    // Mouse leave
    {
        let mouse_x = mouse_x.clone();
        let cb = Closure::wrap(Box::new(move |_: web_sys::Event| {
            *mouse_x.borrow_mut() = -1.0;
        }) as Box<dyn Fn(web_sys::Event)>);
        canvas.add_event_listener_with_callback("mouseleave", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }

    // Pan (drag)
    {
        let graph = graph.clone();
        let canvas_ref = canvas.clone();
        let cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            let rect = canvas_ref.get_bounding_client_rect();
            let scale = CANVAS_W / rect.width();
            let sx = (e.client_x() as f64 - rect.left()) * scale;
            let sy = (e.client_y() as f64 - rect.top()) * scale;
            let mut g = graph.borrow_mut();
            g.dragging = true;
            g.drag_start_x = sx;
            g.drag_start_y = sy;
            g.drag_center_x = g.center_x;
            g.drag_center_y = g.center_y;
        }) as Box<dyn Fn(web_sys::MouseEvent)>);
        canvas.add_event_listener_with_callback("mousedown", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }
    {
        let graph = graph.clone();
        let canvas_ref = canvas.clone();
        let cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            let mut g = graph.borrow_mut();
            if !g.dragging { return; }
            let rect = canvas_ref.get_bounding_client_rect();
            let scale = CANVAS_W / rect.width();
            let sx = (e.client_x() as f64 - rect.left()) * scale;
            let sy = (e.client_y() as f64 - rect.top()) * scale;
            g.center_x = g.drag_center_x - (sx - g.drag_start_x) / g.zoom;
            g.center_y = g.drag_center_y + (sy - g.drag_start_y) / g.zoom;
        }) as Box<dyn Fn(web_sys::MouseEvent)>);
        doc.add_event_listener_with_callback("mousemove", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }
    {
        let graph = graph.clone();
        let cb = Closure::wrap(Box::new(move |_: web_sys::Event| {
            graph.borrow_mut().dragging = false;
        }) as Box<dyn Fn(web_sys::Event)>);
        doc.add_event_listener_with_callback("mouseup", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }

    // Zoom (wheel)
    {
        let graph = graph.clone();
        let canvas_ref = canvas.clone();
        let cb = Closure::wrap(Box::new(move |e: web_sys::Event| {
            e.prevent_default();
            let we: &web_sys::WheelEvent = e.unchecked_ref();
            let rect = canvas_ref.get_bounding_client_rect();
            let scale = CANVAS_W / rect.width();
            let sx = (we.client_x() as f64 - rect.left()) * scale;
            let sy = (we.client_y() as f64 - rect.top()) * scale;
            let mut g = graph.borrow_mut();
            let mx = g.screen_to_math_x(sx);
            let my = g.screen_to_math_y(sy);
            let factor = if we.delta_y() > 0.0 { 0.9 } else { 1.1 };
            g.zoom *= factor;
            g.zoom = g.zoom.clamp(5.0, 2000.0);
            // Zoom toward cursor
            g.center_x = mx - (sx - CANVAS_W / 2.0) / g.zoom;
            g.center_y = my + (sy - CANVAS_H / 2.0) / g.zoom;
        }) as Box<dyn Fn(web_sys::Event)>);
        canvas.add_event_listener_with_callback("wheel", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }

    // Animation loop
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        draw_graph(&ctx, &graph.borrow(), &exprs.borrow(), *mouse_x.borrow());

        web_sys::window().unwrap()
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }) as Box<dyn FnMut()>));

    web_sys::window().unwrap()
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();
}
