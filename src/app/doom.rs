//! ox∅ Doom — raycasting FPS with enemies, shooting, health, and pickups.

use crate::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::cell::RefCell;
use std::rc::Rc;

const W: usize = 640;
const H: usize = 400;
const MAP_W: usize = 16;
const MAP_H: usize = 16;
const FOV: f64 = std::f64::consts::PI / 3.0;
const MAX_DIST: f64 = 20.0;
const MOVE_SPEED: f64 = 0.06;
const ROT_SPEED: f64 = 0.04;

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

fn wall_color(wall: u8, side: bool) -> (u8, u8, u8) {
    let (r, g, b) = match wall {
        1 => (140, 140, 150), 2 => (180, 60, 60), 3 => (60, 140, 180),
        4 => (80, 160, 80), 5 => (180, 140, 60), _ => (100, 100, 100),
    };
    if side { (r * 3/4, g * 3/4, b * 3/4) } else { (r, g, b) }
}

// ── Enemy ─────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum EnemyState { Idle, Chase, Hurt, Dead }

#[derive(Clone)]
struct Enemy {
    x: f64,
    y: f64,
    hp: i32,
    state: EnemyState,
    kind: u8,          // 0=imp, 1=demon, 2=baron
    hurt_timer: f64,
    speed: f64,
    damage: i32,
    attack_cooldown: f64,
    last_attack: f64,
}

impl Enemy {
    fn new(x: f64, y: f64, kind: u8) -> Self {
        let (hp, speed, damage) = match kind {
            0 => (30, 0.02, 5),   // imp — fast, weak
            1 => (60, 0.015, 10), // demon — medium
            _ => (100, 0.01, 20), // baron — slow, strong
        };
        Self {
            x, y, hp, state: EnemyState::Idle, kind,
            hurt_timer: 0.0, speed, damage,
            attack_cooldown: 0.8, last_attack: 0.0,
        }
    }

    fn color(&self) -> (u8, u8, u8) {
        if self.state == EnemyState::Hurt { return (255, 255, 255); }
        match self.kind {
            0 => (200, 80, 40),   // imp — orange
            1 => (180, 40, 60),   // demon — dark red
            _ => (60, 200, 80),   // baron — green
        }
    }

    fn size(&self) -> f64 {
        match self.kind { 0 => 0.4, 1 => 0.5, _ => 0.6 }
    }
}

// ── Game State ────────────────────────────────────────────────────

struct Player {
    x: f64,
    y: f64,
    angle: f64,
    hp: i32,
    max_hp: i32,
    kills: u32,
    shoot_timer: f64,
    damage_flash: f64,
}

struct Input {
    forward: bool, backward: bool, left: bool, right: bool,
    strafe_left: bool, strafe_right: bool,
    mouse_dx: f64, shooting: bool,
}

struct Game {
    player: Player,
    enemies: Vec<Enemy>,
    z_buffer: Vec<f64>,   // per-column wall distance for sprite clipping
    now: f64,
}

impl Game {
    fn new() -> Self {
        let enemies = vec![
            // Imps
            Enemy::new(4.5, 2.5, 0),
            Enemy::new(12.5, 4.5, 0),
            Enemy::new(2.5, 10.5, 0),
            Enemy::new(13.5, 13.5, 0),
            // Demons
            Enemy::new(7.5, 7.5, 1),
            Enemy::new(10.5, 2.5, 1),
            Enemy::new(5.5, 13.5, 1),
            // Baron
            Enemy::new(13.0, 10.0, 2),
        ];
        Self {
            player: Player {
                x: 8.0, y: 14.0, angle: -std::f64::consts::FRAC_PI_2,
                hp: 100, max_hp: 100, kills: 0,
                shoot_timer: 0.0, damage_flash: 0.0,
            },
            enemies,
            z_buffer: vec![0.0; W],
            now: 0.0,
        }
    }

    fn update(&mut self, input: &Input) {
        self.now = js_sys::Date::now() / 1000.0;

        if self.player.hp <= 0 { return; }

        // Movement
        self.player.angle += input.mouse_dx * 0.003;
        let (ca, sa) = (self.player.angle.cos(), self.player.angle.sin());
        let (mut dx, mut dy) = (0.0, 0.0);
        if input.forward  { dx += ca * MOVE_SPEED; dy += sa * MOVE_SPEED; }
        if input.backward { dx -= ca * MOVE_SPEED; dy -= sa * MOVE_SPEED; }
        if input.strafe_left  { dx += sa * MOVE_SPEED; dy -= ca * MOVE_SPEED; }
        if input.strafe_right { dx -= sa * MOVE_SPEED; dy += ca * MOVE_SPEED; }
        if input.left  { self.player.angle -= ROT_SPEED; }
        if input.right { self.player.angle += ROT_SPEED; }

        let m = 0.2;
        if map_at((self.player.x + dx + m * dx.signum()) as usize, self.player.y as usize) == 0 { self.player.x += dx; }
        if map_at(self.player.x as usize, (self.player.y + dy + m * dy.signum()) as usize) == 0 { self.player.y += dy; }

        // Shooting
        let should_shoot = input.shooting && self.player.shoot_timer <= 0.0;
        if should_shoot { self.player.shoot_timer = 0.3; }
        self.player.shoot_timer = (self.player.shoot_timer - 0.016).max(0.0);
        self.player.damage_flash = (self.player.damage_flash - 0.05).max(0.0);

        if should_shoot { self.shoot(); }

        // Update enemies
        let px = self.player.x;
        let py = self.player.y;
        let now = self.now;

        for e in &mut self.enemies {
            if e.state == EnemyState::Dead { continue; }

            // Hurt timer
            if e.hurt_timer > 0.0 {
                e.hurt_timer -= 0.016;
                if e.hurt_timer <= 0.0 { e.state = EnemyState::Chase; }
            }

            let edx = px - e.x;
            let edy = py - e.y;
            let dist = (edx * edx + edy * edy).sqrt();

            // Detection range
            if dist < 10.0 || e.state == EnemyState::Chase {
                e.state = if e.state == EnemyState::Hurt { EnemyState::Hurt } else { EnemyState::Chase };

                // Move toward player
                if dist > 0.8 && e.state == EnemyState::Chase {
                    let nx = edx / dist * e.speed;
                    let ny = edy / dist * e.speed;
                    let new_x = e.x + nx;
                    let new_y = e.y + ny;
                    if map_at(new_x as usize, e.y as usize) == 0 { e.x = new_x; }
                    if map_at(e.x as usize, new_y as usize) == 0 { e.y = new_y; }
                }

                // Attack
                if dist < 1.2 && now - e.last_attack > e.attack_cooldown {
                    e.last_attack = now;
                    self.player.hp -= e.damage;
                    self.player.damage_flash = 1.0;
                }
            }
        }
    }

    fn shoot(&mut self) {
        // Hitscan — find closest enemy near crosshair
        let p = &self.player;
        let mut best_dist = MAX_DIST;
        let mut best_idx: Option<usize> = None;

        for (i, e) in self.enemies.iter().enumerate() {
            if e.state == EnemyState::Dead { continue; }

            let dx = e.x - p.x;
            let dy = e.y - p.y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist >= best_dist { continue; }

            // Angle to enemy
            let angle_to = dy.atan2(dx);
            let mut diff = angle_to - p.angle;
            while diff > std::f64::consts::PI { diff -= std::f64::consts::TAU; }
            while diff < -std::f64::consts::PI { diff += std::f64::consts::TAU; }

            // Check if in crosshair cone (wider for closer enemies)
            let spread = (e.size() / dist).atan();
            if diff.abs() < spread + 0.05 {
                // Check line of sight
                let wall_dist = cast_ray(p.x, p.y, angle_to).dist;
                if dist < wall_dist {
                    best_dist = dist;
                    best_idx = Some(i);
                }
            }
        }

        if let Some(i) = best_idx {
            let e = &mut self.enemies[i];
            let dmg = (40.0 / best_dist.max(1.0)) as i32 + 10;
            e.hp -= dmg;
            e.hurt_timer = 0.15;
            e.state = if e.hp <= 0 {
                self.player.kills += 1;
                EnemyState::Dead
            } else {
                EnemyState::Hurt
            };
        }
    }
}

// ── Raycasting ────────────────────────────────────────────────────

struct HitResult { dist: f64, wall: u8, side: bool, tex_x: f64 }

fn cast_ray(px: f64, py: f64, angle: f64) -> HitResult {
    let (dir_x, dir_y) = (angle.cos(), angle.sin());
    let (mut mx, mut my) = (px as i32, py as i32);
    let dx = if dir_x == 0.0 { 1e30 } else { (1.0 / dir_x).abs() };
    let dy = if dir_y == 0.0 { 1e30 } else { (1.0 / dir_y).abs() };
    let (sx, mut sdx) = if dir_x < 0.0 { (-1, (px - mx as f64) * dx) } else { (1, (mx as f64 + 1.0 - px) * dx) };
    let (sy, mut sdy) = if dir_y < 0.0 { (-1, (py - my as f64) * dy) } else { (1, (my as f64 + 1.0 - py) * dy) };

    let mut side = false;
    loop {
        if sdx < sdy { sdx += dx; mx += sx; side = false; }
        else { sdy += dy; my += sy; side = true; }
        let wall = map_at(mx as usize, my as usize);
        if wall > 0 {
            let dist = if !side { (mx as f64 - px + (1.0 - sx as f64) / 2.0) / dir_x }
                       else { (my as f64 - py + (1.0 - sy as f64) / 2.0) / dir_y };
            let tex_x = if !side { let h = py + dist * dir_y; h - h.floor() }
                        else { let h = px + dist * dir_x; h - h.floor() };
            return HitResult { dist: dist.max(0.001), wall, side, tex_x };
        }
        if sdx > MAX_DIST && sdy > MAX_DIST {
            return HitResult { dist: MAX_DIST, wall: 0, side: false, tex_x: 0.0 };
        }
    }
}

// ── Rendering ─────────────────────────────────────────────────────

fn render(pixels: &mut [u8], game: &mut Game) {
    let p = &game.player;
    let half_h = H as f64 / 2.0;

    // Walls + z-buffer
    for x in 0..W {
        let ray_angle = p.angle - FOV / 2.0 + (x as f64 / W as f64) * FOV;
        let hit = cast_ray(p.x, p.y, ray_angle);
        let perp = hit.dist * (ray_angle - p.angle).cos();
        game.z_buffer[x] = perp;

        let wh = (H as f64 / perp).min(H as f64 * 4.0);
        let wt = ((half_h - wh / 2.0) as usize).min(H);
        let wb = ((half_h + wh / 2.0) as usize).min(H);
        let (wr, wg, wb_c) = if hit.wall > 0 { wall_color(hit.wall, hit.side) } else { (30, 30, 40) };

        for y in 0..H {
            let idx = (y * W + x) * 4;
            if y < wt {
                let f = 1.0 - (y as f64 / half_h);
                let c = (30.0 * f) as u8;
                set_pixel(pixels, idx, c/3, c/3, c);
            } else if y < wb {
                let stripe = ((hit.tex_x * 8.0) as u32 % 2 == 0) as u8;
                let shade = 1.0 - (perp / MAX_DIST).min(1.0) * 0.7;
                let s = stripe as f64 * 0.05 + 0.95;
                set_pixel(pixels, idx, ((wr as f64)*shade*s) as u8, ((wg as f64)*shade*s) as u8, ((wb_c as f64)*shade*s) as u8);
            } else {
                let f = (y as f64 - half_h) / half_h;
                let c = (40.0 * f) as u8;
                set_pixel(pixels, idx, c/3, c/2, c/3);
            }
        }
    }

    // Sprites (sorted back-to-front)
    let mut sprite_order: Vec<(usize, f64)> = game.enemies.iter().enumerate()
        .filter(|(_, e)| e.state != EnemyState::Dead || e.hurt_timer > -0.5)
        .map(|(i, e)| {
            let dx = e.x - p.x;
            let dy = e.y - p.y;
            (i, dx * dx + dy * dy)
        })
        .collect();
    sprite_order.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    for (i, _) in sprite_order {
        let e = &game.enemies[i];
        let dx = e.x - p.x;
        let dy = e.y - p.y;
        let dist = (dx * dx + dy * dy).sqrt();

        // Transform to camera space
        let inv = 1.0 / (p.angle.cos() * p.angle.sin() - p.angle.sin() * p.angle.cos() + 0.0001);
        // Simpler: angle from player to sprite
        let angle_to = dy.atan2(dx);
        let mut rel = angle_to - p.angle;
        while rel > std::f64::consts::PI { rel -= std::f64::consts::TAU; }
        while rel < -std::f64::consts::PI { rel += std::f64::consts::TAU; }

        // Screen position
        let screen_x = (W as f64 / 2.0) * (1.0 + rel / (FOV / 2.0));
        let sprite_h = (H as f64 / dist).min(H as f64 * 2.0);
        let sprite_w = sprite_h * 0.6;

        let (cr, cg, cb) = e.color();
        let shade = (1.0 - (dist / MAX_DIST).min(1.0) * 0.7) as f64;

        // Dead enemies are flat
        let h_mult = if e.state == EnemyState::Dead { 0.2 } else { 1.0 };
        let y_offset = if e.state == EnemyState::Dead { sprite_h * 0.4 } else { 0.0 };

        let left = (screen_x - sprite_w / 2.0) as i32;
        let right = (screen_x + sprite_w / 2.0) as i32;
        let top = ((H as f64 / 2.0 - sprite_h * h_mult / 2.0 + y_offset) as i32).max(0);
        let bot = ((H as f64 / 2.0 + sprite_h * h_mult / 2.0 + y_offset) as i32).min(H as i32);

        for sx in left.max(0)..right.min(W as i32) {
            // Z-buffer test
            if dist >= game.z_buffer[sx as usize] { continue; }

            let tx = (sx - left) as f64 / (right - left) as f64;

            for sy in top..bot {
                let ty = (sy - top) as f64 / (bot - top) as f64;

                // Simple body shape: oval with head
                let cx_rel = tx - 0.5;
                let cy_rel = ty - 0.5;

                // Body (ellipse)
                let in_body = cx_rel * cx_rel * 4.0 + cy_rel * cy_rel * 2.5 < 0.5;
                // Head (circle at top)
                let head_y = cy_rel + 0.3;
                let in_head = cx_rel * cx_rel + head_y * head_y < 0.06;
                // Eyes
                let eye_l = (cx_rel + 0.08) * (cx_rel + 0.08) + (head_y + 0.02) * (head_y + 0.02);
                let eye_r = (cx_rel - 0.08) * (cx_rel - 0.08) + (head_y + 0.02) * (head_y + 0.02);
                let in_eye = eye_l < 0.004 || eye_r < 0.004;

                if in_body || in_head {
                    let idx = (sy as usize * W + sx as usize) * 4;
                    if in_eye {
                        set_pixel(pixels, idx, 255, 50, 50);
                    } else {
                        let sr = ((cr as f64) * shade) as u8;
                        let sg = ((cg as f64) * shade) as u8;
                        let sb = ((cb as f64) * shade) as u8;
                        set_pixel(pixels, idx, sr, sg, sb);
                    }
                }
            }
        }
    }

    // Damage flash
    if game.player.damage_flash > 0.0 {
        let alpha = (game.player.damage_flash * 0.3).min(0.3);
        for i in (0..W * H * 4).step_by(4) {
            pixels[i] = (pixels[i] as f64 + (255.0 - pixels[i] as f64) * alpha) as u8;
            pixels[i+1] = (pixels[i+1] as f64 * (1.0 - alpha)) as u8;
            pixels[i+2] = (pixels[i+2] as f64 * (1.0 - alpha)) as u8;
        }
    }

    // Shoot flash
    if game.player.shoot_timer > 0.2 {
        let flash = (game.player.shoot_timer - 0.2) / 0.1;
        let cx = W / 2;
        let cy = H / 2 + 20;
        let r = (30.0 * flash) as usize;
        for dy in 0..r {
            for dx in 0..r {
                let d = ((dx * dx + dy * dy) as f64).sqrt();
                if d < r as f64 {
                    let bright = (1.0 - d / r as f64) * flash;
                    for (ox, oy) in [(cx+dx, cy+dy), (cx.wrapping_sub(dx), cy+dy), (cx+dx, cy.wrapping_sub(dy)), (cx.wrapping_sub(dx), cy.wrapping_sub(dy))] {
                        if ox < W && oy < H {
                            let idx = (oy * W + ox) * 4;
                            pixels[idx] = (pixels[idx] as f64 + (255.0 - pixels[idx] as f64) * bright) as u8;
                            pixels[idx+1] = (pixels[idx+1] as f64 + (200.0 - pixels[idx+1] as f64) * bright * 0.5) as u8;
                        }
                    }
                }
            }
        }
    }
}

fn render_minimap(pixels: &mut [u8], game: &Game) {
    let scale = 4;
    let ox = W - MAP_W * scale - 8;
    let oy = 8;
    let p = &game.player;

    for my in 0..MAP_H {
        for mx in 0..MAP_W {
            let wall = map_at(mx, my);
            let (r, g, b) = if wall > 0 { wall_color(wall, false) } else { (20, 22, 30) };
            for dy in 0..scale {
                for dx in 0..scale {
                    let px = ox + mx * scale + dx;
                    let py = oy + my * scale + dy;
                    if px < W && py < H {
                        let idx = (py * W + px) * 4;
                        set_pixel(pixels, idx, r, g, b);
                    }
                }
            }
        }
    }

    // Enemy dots on minimap
    for e in &game.enemies {
        if e.state == EnemyState::Dead { continue; }
        let ex = ox + (e.x * scale as f64) as usize;
        let ey = oy + (e.y * scale as f64) as usize;
        let (cr, cg, cb) = e.color();
        for dy in 0..2usize {
            for dx in 0..2usize {
                if ex+dx < W && ey+dy < H {
                    let idx = ((ey+dy) * W + (ex+dx)) * 4;
                    set_pixel(pixels, idx, cr, cg, cb);
                }
            }
        }
    }

    // Player dot
    let ppx = ox + (p.x * scale as f64) as usize;
    let ppy = oy + (p.y * scale as f64) as usize;
    for dy in 0..3usize {
        for dx in 0..3usize {
            if ppx+dx < W && ppy+dy < H {
                let idx = ((ppy+dy) * W + (ppx+dx)) * 4;
                set_pixel(pixels, idx, 80, 200, 255);
            }
        }
    }
}

fn render_hud(ctx: &web_sys::CanvasRenderingContext2d, game: &Game) {
    let p = &game.player;

    // Health bar background
    ctx.set_fill_style_str("rgba(0,0,0,0.6)");
    ctx.fill_rect(10.0, (H as f64) - 40.0, 200.0, 24.0);

    // Health bar
    let hp_pct = (p.hp as f64 / p.max_hp as f64).max(0.0);
    let color = if hp_pct > 0.5 { "#40c057" } else if hp_pct > 0.25 { "#fab005" } else { "#fa5252" };
    ctx.set_fill_style_str(color);
    ctx.fill_rect(12.0, (H as f64) - 38.0, 196.0 * hp_pct, 20.0);

    // HP text
    ctx.set_fill_style_str("#fff");
    ctx.set_font("bold 14px 'JetBrains Mono', monospace");
    ctx.set_text_align("left");
    ctx.fill_text(&format!("HP {}/{}", p.hp.max(0), p.max_hp), 16.0, (H as f64) - 22.0).ok();

    // Kills
    ctx.set_text_align("right");
    ctx.fill_text(&format!("KILLS {}", p.kills), (W as f64) - 16.0, (H as f64) - 22.0).ok();

    // Crosshair
    ctx.set_stroke_style_str("rgba(255,255,255,0.5)");
    ctx.set_line_width(1.5);
    let cx = W as f64 / 2.0;
    let cy = H as f64 / 2.0;
    ctx.begin_path();
    ctx.move_to(cx - 12.0, cy); ctx.line_to(cx - 4.0, cy);
    ctx.move_to(cx + 4.0, cy); ctx.line_to(cx + 12.0, cy);
    ctx.move_to(cx, cy - 12.0); ctx.line_to(cx, cy - 4.0);
    ctx.move_to(cx, cy + 4.0); ctx.line_to(cx, cy + 12.0);
    ctx.stroke();

    // Game over
    if p.hp <= 0 {
        ctx.set_global_alpha(0.7);
        ctx.set_fill_style_str("#0a0c10");
        ctx.fill_rect(0.0, 0.0, W as f64, H as f64);
        ctx.set_global_alpha(1.0);
        ctx.set_fill_style_str("#fa5252");
        ctx.set_font("bold 36px 'Outfit', sans-serif");
        ctx.set_text_align("center");
        ctx.fill_text("YOU DIED", cx, cy - 20.0).ok();
        ctx.set_fill_style_str("#e4e5e9");
        ctx.set_font("bold 20px 'JetBrains Mono', monospace");
        ctx.fill_text(&format!("Kills: {}", p.kills), cx, cy + 20.0).ok();
        ctx.set_fill_style_str("#9ca0ab");
        ctx.set_font("14px 'DM Sans', sans-serif");
        ctx.fill_text("Press R to restart", cx, cy + 55.0).ok();
    }
}

#[inline(always)]
fn set_pixel(pixels: &mut [u8], idx: usize, r: u8, g: u8, b: u8) {
    pixels[idx] = r; pixels[idx+1] = g; pixels[idx+2] = b; pixels[idx+3] = 255;
}

// ── Entry Point ───────────────────────────────────────────────────

pub fn run() {
    console_error_panic_hook::set_once();

    let doc = document();
    let root = doc.get_element_by_id("app").unwrap();

    let container = el("div", "");
    attr(&container, "style", "display:flex;flex-direction:column;align-items:center;padding:1rem");

    let title = text_el("h1", "ox-h3 ox-font-display ox-text-center ox-mb-2", "ox∅ Doom");
    let sub = text_el("p", "ox-text-center ox-mb-3",
        "Click to capture mouse. WASD move, mouse look, click to shoot. Kill all 8 enemies.");
    attr(&sub, "style", "color:#9ca0ab;font-size:0.875rem");

    let canvas = doc.create_element("canvas").unwrap();
    canvas.set_attribute("width", &W.to_string()).unwrap();
    canvas.set_attribute("height", &H.to_string()).unwrap();
    canvas.set_attribute("style", &format!(
        "width:{}px;height:{}px;display:block;border-radius:0.75rem;\
         border:1px solid #2e3140;cursor:crosshair", W, H)).unwrap();

    append(&container, &[&title, &sub, &canvas]);
    root.append_child(&container).unwrap();

    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
    let ctx = canvas.get_context("2d").unwrap().unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();

    let game = Rc::new(RefCell::new(Game::new()));
    let input = Rc::new(RefCell::new(Input {
        forward: false, backward: false, left: false, right: false,
        strafe_left: false, strafe_right: false, mouse_dx: 0.0, shooting: false,
    }));

    // Pointer lock
    {
        let canvas_ref = canvas.clone();
        let cb = Closure::wrap(Box::new(move |_: web_sys::Event| {
            canvas_ref.request_pointer_lock();
        }) as Box<dyn Fn(web_sys::Event)>);
        canvas.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }

    // Mouse
    {
        let input = input.clone();
        let cb = Closure::wrap(Box::new(move |e: web_sys::Event| {
            let me: &web_sys::MouseEvent = e.unchecked_ref();
            let mut inp = input.borrow_mut();
            inp.mouse_dx += me.movement_x() as f64;
            if me.buttons() & 1 != 0 { inp.shooting = true; }
        }) as Box<dyn Fn(web_sys::Event)>);
        doc.add_event_listener_with_callback("mousemove", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }
    {
        let input = input.clone();
        let cb = Closure::wrap(Box::new(move |_: web_sys::Event| {
            input.borrow_mut().shooting = true;
        }) as Box<dyn Fn(web_sys::Event)>);
        doc.add_event_listener_with_callback("mousedown", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }

    // Keyboard
    {
        let input = input.clone();
        let game2 = game.clone();
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
                "r" | "R" if pressed    => { *game2.borrow_mut() = Game::new(); },
                _ => {}
            }
            e.prevent_default();
        }) as Box<dyn Fn(web_sys::Event)>);
        doc.add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref()).unwrap();
        doc.add_event_listener_with_callback("keyup", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    }

    let mut pixels = vec![0u8; W * H * 4];

    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        {
            let inp = input.borrow();
            game.borrow_mut().update(&inp);
        }
        input.borrow_mut().mouse_dx = 0.0;
        input.borrow_mut().shooting = false;

        render(&mut pixels, &mut game.borrow_mut());
        render_minimap(&mut pixels, &game.borrow());

        let data = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(&pixels), W as u32, H as u32
        ).unwrap();
        ctx.put_image_data(&data, 0.0, 0.0).unwrap();
        render_hud(&ctx, &game.borrow());

        web_sys::window().unwrap()
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }) as Box<dyn FnMut()>));

    web_sys::window().unwrap()
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();
}
