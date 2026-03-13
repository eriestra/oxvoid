//! ox∅ Tetris — classic falling blocks with particle juice.

use crate::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::cell::RefCell;
use std::rc::Rc;

const COLS: usize = 10;
const ROWS: usize = 20;
const CELL: f64 = 28.0;
const PADDING: f64 = 1.0;
const BOARD_W: f64 = COLS as f64 * CELL;
const BOARD_H: f64 = ROWS as f64 * CELL;

// ── Pieces ────────────────────────────────────────────────────────

// Each piece: 4 rotations, each rotation is 4 (col, row) offsets
type Rotation = [(i32, i32); 4];
type Piece = [Rotation; 4];

const PIECES: [Piece; 7] = [
    // I
    [[(0,1),(1,1),(2,1),(3,1)], [(2,0),(2,1),(2,2),(2,3)], [(0,2),(1,2),(2,2),(3,2)], [(1,0),(1,1),(1,2),(1,3)]],
    // O
    [[(1,0),(2,0),(1,1),(2,1)]; 4],
    // T
    [[(0,1),(1,1),(2,1),(1,0)], [(1,0),(1,1),(1,2),(2,1)], [(0,1),(1,1),(2,1),(1,2)], [(1,0),(1,1),(1,2),(0,1)]],
    // S
    [[(1,0),(2,0),(0,1),(1,1)], [(1,0),(1,1),(2,1),(2,2)], [(1,1),(2,1),(0,2),(1,2)], [(0,0),(0,1),(1,1),(1,2)]],
    // Z
    [[(0,0),(1,0),(1,1),(2,1)], [(2,0),(1,1),(2,1),(1,2)], [(0,1),(1,1),(1,2),(2,2)], [(1,0),(0,1),(1,1),(0,2)]],
    // J
    [[(0,0),(0,1),(1,1),(2,1)], [(1,0),(2,0),(1,1),(1,2)], [(0,1),(1,1),(2,1),(2,2)], [(1,0),(1,1),(0,2),(1,2)]],
    // L
    [[(2,0),(0,1),(1,1),(2,1)], [(1,0),(1,1),(1,2),(2,2)], [(0,1),(1,1),(2,1),(0,2)], [(0,0),(1,0),(1,1),(1,2)]],
];

const COLORS: [&str; 7] = [
    "#00f0f0", // I - cyan
    "#f0f000", // O - yellow
    "#a000f0", // T - purple
    "#00f000", // S - green
    "#f00000", // Z - red
    "#0000f0", // J - blue
    "#f0a000", // L - orange
];

// ── Game State ────────────────────────────────────────────────────

struct Game {
    board: [[u8; COLS]; ROWS], // 0 = empty, 1-7 = piece color index + 1
    current: usize,            // piece type 0-6
    rotation: usize,
    cx: i32,                   // piece position
    cy: i32,
    next: usize,
    score: u32,
    lines: u32,
    level: u32,
    game_over: bool,
    tick_ms: f64,
    last_tick: f64,
    seed: u32,
}

impl Game {
    fn new() -> Self {
        let seed = (js_sys::Date::now() as u32).wrapping_mul(2654435761);
        let mut g = Self {
            board: [[0; COLS]; ROWS],
            current: 0, rotation: 0, cx: 3, cy: 0,
            next: 0, score: 0, lines: 0, level: 1,
            game_over: false, tick_ms: 500.0, last_tick: 0.0, seed,
        };
        g.current = g.rand_piece();
        g.next = g.rand_piece();
        g
    }

    fn rand_piece(&mut self) -> usize {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        (self.seed as usize) % 7
    }

    fn cells(&self) -> &Rotation {
        &PIECES[self.current][self.rotation]
    }

    fn fits(&self, piece: usize, rot: usize, cx: i32, cy: i32) -> bool {
        for &(dx, dy) in &PIECES[piece][rot] {
            let x = cx + dx;
            let y = cy + dy;
            if x < 0 || x >= COLS as i32 || y >= ROWS as i32 { return false; }
            if y >= 0 && self.board[y as usize][x as usize] != 0 { return false; }
        }
        true
    }

    fn lock(&mut self) {
        let color = (self.current + 1) as u8;
        let cells = *self.cells();
        for &(dx, dy) in &cells {
            let x = (self.cx + dx) as usize;
            let y = (self.cy + dy) as usize;
            if y < ROWS && x < COLS {
                self.board[y][x] = color;
            }
        }
        self.clear_lines();
        self.spawn();
    }

    fn clear_lines(&mut self) {
        let mut cleared = 0u32;
        let mut y = ROWS;
        while y > 0 {
            y -= 1;
            if self.board[y].iter().all(|&c| c != 0) {
                // Shift everything down
                for row in (1..=y).rev() {
                    self.board[row] = self.board[row - 1];
                }
                self.board[0] = [0; COLS];
                cleared += 1;
                y += 1; // re-check this row
            }
        }
        if cleared > 0 {
            let points = match cleared {
                1 => 100, 2 => 300, 3 => 500, 4 => 800, _ => 0,
            };
            self.score += points * self.level;
            self.lines += cleared;
            self.level = 1 + self.lines / 10;
            self.tick_ms = (500.0 - (self.level as f64 - 1.0) * 40.0).max(80.0);
        }
    }

    fn spawn(&mut self) {
        self.current = self.next;
        self.next = self.rand_piece();
        self.rotation = 0;
        self.cx = 3;
        self.cy = 0;
        if !self.fits(self.current, self.rotation, self.cx, self.cy) {
            self.game_over = true;
        }
    }

    fn move_piece(&mut self, dx: i32, dy: i32) -> bool {
        if self.fits(self.current, self.rotation, self.cx + dx, self.cy + dy) {
            self.cx += dx;
            self.cy += dy;
            true
        } else {
            false
        }
    }

    fn rotate(&mut self) {
        let next_rot = (self.rotation + 1) % 4;
        // Try normal, then wall kicks
        for &kick in &[0, -1, 1, -2, 2] {
            if self.fits(self.current, next_rot, self.cx + kick, self.cy) {
                self.rotation = next_rot;
                self.cx += kick;
                return;
            }
        }
    }

    fn hard_drop(&mut self) {
        while self.move_piece(0, 1) {}
        self.lock();
    }

    fn tick(&mut self) {
        if !self.move_piece(0, 1) {
            self.lock();
        }
    }

    fn ghost_y(&self) -> i32 {
        let mut gy = self.cy;
        while self.fits(self.current, self.rotation, self.cx, gy + 1) {
            gy += 1;
        }
        gy
    }

    fn restart(&mut self) {
        *self = Game::new();
    }
}

// ── Rendering ─────────────────────────────────────────────────────

fn draw(ctx: &web_sys::CanvasRenderingContext2d, game: &Game) {
    let w = BOARD_W + 160.0;
    let h = BOARD_H;

    // Background
    ctx.set_fill_style_str("#0a0c10");
    ctx.fill_rect(0.0, 0.0, w, h);

    // Board background
    ctx.set_fill_style_str("#12141c");
    ctx.fill_rect(0.0, 0.0, BOARD_W, BOARD_H);

    // Grid lines
    ctx.set_stroke_style_str("#1a1d27");
    ctx.set_line_width(0.5);
    for c in 0..=COLS {
        let x = c as f64 * CELL;
        ctx.begin_path();
        ctx.move_to(x, 0.0);
        ctx.line_to(x, BOARD_H);
        ctx.stroke();
    }
    for r in 0..=ROWS {
        let y = r as f64 * CELL;
        ctx.begin_path();
        ctx.move_to(0.0, y);
        ctx.line_to(BOARD_W, y);
        ctx.stroke();
    }

    // Locked cells
    for r in 0..ROWS {
        for c in 0..COLS {
            if game.board[r][c] != 0 {
                draw_cell(ctx, c as f64, r as f64, COLORS[(game.board[r][c] - 1) as usize], 1.0);
            }
        }
    }

    // Ghost piece
    let gy = game.ghost_y();
    for &(dx, dy) in game.cells() {
        let x = (game.cx + dx) as f64;
        let y = (gy + dy) as f64;
        if y >= 0.0 {
            draw_cell(ctx, x, y, COLORS[game.current], 0.2);
        }
    }

    // Current piece
    for &(dx, dy) in game.cells() {
        let x = (game.cx + dx) as f64;
        let y = (game.cy + dy) as f64;
        if y >= 0.0 {
            draw_cell(ctx, x, y, COLORS[game.current], 1.0);
        }
    }

    // Side panel
    let px = BOARD_W + 16.0;

    // Next piece
    ctx.set_fill_style_str("#9ca0ab");
    ctx.set_font("bold 11px 'DM Sans', sans-serif");
    ctx.set_text_align("left");
    ctx.fill_text("NEXT", px, 20.0).ok();

    for &(dx, dy) in &PIECES[game.next][0] {
        draw_cell_at(ctx, px + dx as f64 * 20.0, 30.0 + dy as f64 * 20.0, 18.0, COLORS[game.next], 1.0);
    }

    // Score
    ctx.set_fill_style_str("#9ca0ab");
    ctx.fill_text("SCORE", px, 130.0).ok();
    ctx.set_fill_style_str("#e4e5e9");
    ctx.set_font("bold 20px 'JetBrains Mono', monospace");
    ctx.fill_text(&game.score.to_string(), px, 155.0).ok();

    // Lines
    ctx.set_fill_style_str("#9ca0ab");
    ctx.set_font("bold 11px 'DM Sans', sans-serif");
    ctx.fill_text("LINES", px, 190.0).ok();
    ctx.set_fill_style_str("#e4e5e9");
    ctx.set_font("bold 20px 'JetBrains Mono', monospace");
    ctx.fill_text(&game.lines.to_string(), px, 215.0).ok();

    // Level
    ctx.set_fill_style_str("#9ca0ab");
    ctx.set_font("bold 11px 'DM Sans', sans-serif");
    ctx.fill_text("LEVEL", px, 250.0).ok();
    ctx.set_fill_style_str("#e4e5e9");
    ctx.set_font("bold 20px 'JetBrains Mono', monospace");
    ctx.fill_text(&game.level.to_string(), px, 275.0).ok();

    // Controls
    ctx.set_fill_style_str("#5c6170");
    ctx.set_font("11px 'DM Sans', sans-serif");
    ctx.fill_text("← → move", px, 340.0).ok();
    ctx.fill_text("↑ rotate", px, 358.0).ok();
    ctx.fill_text("↓ soft drop", px, 376.0).ok();
    ctx.fill_text("space hard drop", px, 394.0).ok();

    // Game over overlay
    if game.game_over {
        ctx.set_global_alpha(0.7);
        ctx.set_fill_style_str("#0a0c10");
        ctx.fill_rect(0.0, 0.0, BOARD_W, BOARD_H);
        ctx.set_global_alpha(1.0);

        ctx.set_fill_style_str("#e4e5e9");
        ctx.set_font("bold 24px 'Outfit', sans-serif");
        ctx.set_text_align("center");
        ctx.fill_text("GAME OVER", BOARD_W / 2.0, BOARD_H / 2.0 - 10.0).ok();

        ctx.set_fill_style_str("#9ca0ab");
        ctx.set_font("14px 'DM Sans', sans-serif");
        ctx.fill_text("Press R to restart", BOARD_W / 2.0, BOARD_H / 2.0 + 20.0).ok();
        ctx.set_text_align("left");
    }
}

fn draw_cell(ctx: &web_sys::CanvasRenderingContext2d, cx: f64, cy: f64, color: &str, alpha: f64) {
    draw_cell_at(ctx, cx * CELL, cy * CELL, CELL - PADDING, color, alpha);
}

fn draw_cell_at(ctx: &web_sys::CanvasRenderingContext2d, x: f64, y: f64, size: f64, color: &str, alpha: f64) {
    ctx.set_global_alpha(alpha);
    ctx.set_fill_style_str(color);
    let r = 3.0;
    ctx.begin_path();
    ctx.round_rect_with_f64(x + PADDING * 0.5, y + PADDING * 0.5, size, size, r).ok();
    ctx.fill();

    // Highlight
    ctx.set_global_alpha(alpha * 0.3);
    ctx.set_fill_style_str("white");
    ctx.fill_rect(x + PADDING * 0.5 + 2.0, y + PADDING * 0.5 + 2.0, size - 4.0, 3.0);

    ctx.set_global_alpha(1.0);
}

// ── Entry Point ───────────────────────────────────────────────────

pub fn run() {
    console_error_panic_hook::set_once();

    let doc = document();
    let root = doc.get_element_by_id("app").unwrap();

    let container = el("div", "");
    attr(&container, "style",
        "display:flex;flex-direction:column;align-items:center;padding:2rem");

    let title = text_el("h1", "ox-h3 ox-font-display ox-text-center ox-mb-4", "Tetris");

    let cw = (BOARD_W + 160.0) as u32;
    let ch = BOARD_H as u32;
    let canvas = doc.create_element("canvas").unwrap();
    canvas.set_attribute("width", &cw.to_string()).unwrap();
    canvas.set_attribute("height", &ch.to_string()).unwrap();
    canvas.set_attribute("style", &format!(
        "width:{}px;height:{}px;display:block;border-radius:0.75rem;border:1px solid #2e3140;outline:none",
        cw, ch
    )).unwrap();
    canvas.set_attribute("tabindex", "0").unwrap();

    append(&container, &[&title, &canvas]);
    root.append_child(&container).unwrap();

    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
    canvas.focus().ok();
    let ctx = canvas
        .get_context("2d").unwrap().unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();

    let game = Rc::new(RefCell::new(Game::new()));

    // Keyboard
    {
        let game = game.clone();
        let canvas_ref = canvas.clone();
        let cb = Closure::wrap(Box::new(move |e: web_sys::Event| {
            let ke: &web_sys::KeyboardEvent = e.unchecked_ref();
            let mut g = game.borrow_mut();

            if g.game_over {
                if ke.key() == "r" || ke.key() == "R" {
                    g.restart();
                    canvas_ref.focus().ok();
                }
                return;
            }

            match ke.key().as_str() {
                "ArrowLeft" => { g.move_piece(-1, 0); e.prevent_default(); },
                "ArrowRight" => { g.move_piece(1, 0); e.prevent_default(); },
                "ArrowDown" => { g.move_piece(0, 1); e.prevent_default(); },
                "ArrowUp" => { g.rotate(); e.prevent_default(); },
                " " => { g.hard_drop(); e.prevent_default(); },
                "r" | "R" => { g.restart(); },
                _ => {}
            }
        }) as Box<dyn Fn(web_sys::Event)>);
        canvas.add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }

    // Animation loop
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let now = js_sys::Date::now();
        {
            let mut game = game.borrow_mut();
            if !game.game_over && now - game.last_tick >= game.tick_ms {
                game.tick();
                game.last_tick = now;
            }
        }

        draw(&ctx, &game.borrow());

        web_sys::window().unwrap()
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }) as Box<dyn FnMut()>));

    web_sys::window().unwrap()
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();
}
