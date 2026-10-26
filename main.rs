// Bubu Jump — port a Rust/macroquad (fase 1: núcleo jugable)
// Valores y reglas tomados de game.gd (versión Godot 4.7.2).
use macroquad::prelude::*;
use std::f32::consts::{PI, TAU};

const GRAV: f32 = 1500.0;
const JUMP: f32 = -720.0;
const SPRING: f32 = -1150.0;
const TRAMP: f32 = -1500.0;
const REACH: f32 = 140.0;
const SPEED: f32 = 340.0;
const SNOW_K: f32 = 2.6;
const WIND_V: f32 = 170.0;
const MARGIN: f32 = 0.8;
const B_SLACK: f32 = 45.0;
const TIME_LIMIT: f32 = 60.0;
const PX_PER_M: f32 = 10.0;
const DUSK_M: f32 = 500.0;
const NIGHT_M: f32 = 900.0;
const BASE_W: f32 = 360.0;
const BASE_H: f32 = 640.0;

const INK: &str = "#3b2420";
const STARC: [&str; 5] = ["#f3d23c", "#e8504a", "#6cc3ee", "#86d443", "#f7b2ae"];
const MODES: [(&str, &str, &str); 3] = [
    ("classic", "clásico", "sube lo más alto que puedas"),
    ("time", "contrarreloj", "60 s para subir lo más alto posible"),
    ("zen", "zen", "sin enemigos"),
];
// (nombre, gap, monstruos, monedas)
const DIFS: [(&str, f32, f32, f32); 3] = [("fácil", 0.85, 0.5, 1.3), ("normal", 1.0, 1.0, 1.0), ("difícil", 1.1, 1.8, 0.7)];

fn hx(s: &str) -> Color {
    let s = s.trim_start_matches('#');
    let v = |i: usize| u8::from_str_radix(&s[i..i + 2], 16).unwrap_or(255) as f32 / 255.0;
    match s.len() {
        3 => {
            let d = |i: usize| u8::from_str_radix(&s[i..i + 1].repeat(2), 16).unwrap_or(255) as f32 / 255.0;
            Color::new(d(0), d(1), d(2), 1.0)
        }
        8 => Color::new(v(0), v(2), v(4), v(6)),
        _ => Color::new(v(0), v(2), v(4), 1.0),
    }
}
fn lerpc(a: Color, b: Color, t: f32) -> Color {
    Color::new(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t, a.a + (b.a - a.a) * t)
}
fn rnd(a: f32, b: f32) -> f32 {
    rand::gen_range(a, b)
}
fn chance() -> f32 {
    rand::gen_range(0.0, 1.0)
}
fn pick<'a>(a: &[&'a str]) -> &'a str {
    a[rand::gen_range(0, a.len())]
}
fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
fn fposmod(a: f32, b: f32) -> f32 {
    ((a % b) + b) % b
}

// ---------- geometría ----------
fn rr_pts(x: f32, y: f32, w: f32, h: f32, r: f32) -> Vec<Vec2> {
    let r = r.min(w.min(h) / 2.0).max(0.05);
    let cs = [(x + r, y + r, PI), (x + w - r, y + r, PI * 1.5), (x + w - r, y + h - r, 0.0), (x + r, y + h - r, PI * 0.5)];
    let mut p = Vec::with_capacity(20);
    for c in cs {
        for i in 0..=4 {
            let a = c.2 + PI * 0.5 * i as f32 / 4.0;
            p.push(vec2(c.0 + a.cos() * r, c.1 + a.sin() * r));
        }
    }
    p
}
fn ell_pts(cx: f32, cy: f32, a: f32, b: f32) -> Vec<Vec2> {
    let n = ((a.max(b) * 1.5) as i32 + 10).clamp(12, 40);
    (0..n).map(|i| {
        let t = TAU * i as f32 / n as f32;
        vec2(cx + t.cos() * a, cy + t.sin() * b)
    }).collect()
}
fn arc_pts(cx: f32, cy: f32, r: f32, a0: f32, mut a1: f32, n: usize) -> Vec<Vec2> {
    if a1 < a0 {
        a1 += TAU;
    }
    (0..=n).map(|i| {
        let a = a0 + (a1 - a0) * i as f32 / n as f32;
        vec2(cx + a.cos() * r, cy + a.sin() * r)
    }).collect()
}
fn quad_pts(p0: Vec2, c: Vec2, p1: Vec2, n: usize) -> Vec<Vec2> {
    (0..=n).map(|i| {
        let t = i as f32 / n as f32;
        p0.lerp(c, t).lerp(c.lerp(p1, t), t)
    }).collect()
}
fn star_pts(r: f32) -> Vec<Vec2> {
    (0..10).map(|i| {
        let a = i as f32 * PI / 5.0 - PI / 2.0;
        let rr = if i % 2 == 1 { r * 0.45 } else { r };
        vec2(a.cos() * rr, a.sin() * rr)
    }).collect()
}
fn cloud_pts() -> Vec<Vec2> {
    let mut p = arc_pts(-18.0, 4.0, 12.0, PI * 0.5, PI * 1.5, 8);
    p.extend(arc_pts(-4.0, -6.0, 15.0, PI, PI * 1.9, 8));
    p.extend(arc_pts(16.0, 0.0, 12.0, PI * 1.3, PI * 0.5, 8));
    p
}

fn cross(o: Vec2, a: Vec2, b: Vec2) -> f32 {
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
}
// triangulación por recorte de orejas (polígonos simples, cóncavos incluidos)
fn triangulate(pts: &[Vec2]) -> Vec<[Vec2; 3]> {
    let n = pts.len();
    let mut out = Vec::new();
    if n < 3 {
        return out;
    }
    let area: f32 = (0..n).map(|i| { let j = (i + 1) % n; pts[i].x * pts[j].y - pts[j].x * pts[i].y }).sum();
    let mut idx: Vec<usize> = if area > 0.0 { (0..n).collect() } else { (0..n).rev().collect() };
    let mut guard = 0;
    while idx.len() > 3 && guard < n * n {
        guard += 1;
        let m = idx.len();
        let mut cut = false;
        for i in 0..m {
            let (a, b, c) = (pts[idx[(i + m - 1) % m]], pts[idx[i]], pts[idx[(i + 1) % m]]);
            if cross(a, b, c) <= 1e-6 {
                continue;
            }
            let inside = idx.iter().any(|&k| {
                let p = pts[k];
                p != a && p != b && p != c && cross(a, b, p) >= 0.0 && cross(b, c, p) >= 0.0 && cross(c, a, p) >= 0.0
            });
            if inside {
                continue;
            }
            out.push([a, b, c]);
            idx.remove(i);
            cut = true;
            break;
        }
        if !cut {
            break;
        }
    }
    if idx.len() >= 3 {
        for i in 1..idx.len() - 1 {
            out.push([pts[idx[0]], pts[idx[i]], pts[idx[i + 1]]]);
        }
    }
    out
}

// ---------- pintor con pila de transformaciones (como gsave/gtr/grot/gsc) ----------
struct Painter {
    m: Affine2,
    stack: Vec<(Affine2, f32)>,
    ga: f32,
    font: Font,
    px: f32,
}
impl Painter {
    fn save(&mut self) {
        self.stack.push((self.m, self.ga));
    }
    fn restore(&mut self) {
        if let Some((m, g)) = self.stack.pop() {
            self.m = m;
            self.ga = g;
        }
    }
    fn tr(&mut self, x: f32, y: f32) {
        self.m = self.m * Affine2::from_translation(vec2(x, y));
    }
    fn rot(&mut self, a: f32) {
        self.m = self.m * Affine2::from_angle(a);
    }
    fn sc(&mut self, x: f32, y: f32) {
        self.m = self.m * Affine2::from_scale(vec2(x, y));
    }
    fn k(&self, c: &str) -> Color {
        let mut c = hx(c);
        c.a *= self.ga;
        c
    }
    fn kc(&self, mut c: Color) -> Color {
        c.a *= self.ga;
        c
    }
    fn tp(&self, p: Vec2) -> Vec2 {
        self.m.transform_point2(p)
    }
    fn lscale(&self) -> f32 {
        self.m.matrix2.determinant().abs().sqrt()
    }
    fn fill_c(&self, pts: &[Vec2], c: Color) {
        if pts.len() < 3 || c.a <= 0.0 {
            return;
        }
        let t: Vec<Vec2> = pts.iter().map(|p| self.tp(*p)).collect();
        for tri in triangulate(&t) {
            draw_triangle(tri[0], tri[1], tri[2], c);
        }
    }
    fn fill(&self, pts: &[Vec2], c: &str) {
        self.fill_c(pts, self.k(c));
    }
    fn stroke_c(&self, pts: &[Vec2], lw: f32, c: Color, closed: bool) {
        if pts.len() < 2 || c.a <= 0.0 {
            return;
        }
        let w = lw * self.lscale();
        let t: Vec<Vec2> = pts.iter().map(|p| self.tp(*p)).collect();
        let n = if closed { t.len() } else { t.len() - 1 };
        for i in 0..n {
            let (a, b) = (t[i], t[(i + 1) % t.len()]);
            draw_line(a.x, a.y, b.x, b.y, w, c);
        }
        if w > 1.4 {
            for p in &t {
                draw_circle(p.x, p.y, w * 0.5, c);
            }
        }
    }
    fn stroke(&self, pts: &[Vec2], lw: f32, c: &str, closed: bool) {
        self.stroke_c(pts, lw, self.k(c), closed);
    }
    fn fe(&self, cx: f32, cy: f32, a: f32, b: f32, c: &str, lw: f32) {
        let p = ell_pts(cx, cy, a, b);
        self.fill(&p, c);
        if lw > 0.0 {
            self.stroke(&p, lw, INK, true);
        }
    }
    fn fcirc(&self, cx: f32, cy: f32, r: f32, c: &str, lw: f32) {
        self.fe(cx, cy, r, r, c, lw);
    }
    fn fr(&self, x: f32, y: f32, w: f32, h: f32, r: f32, c: &str, lw: f32) {
        if w < 0.5 || h < 0.5 {
            return;
        }
        let p = rr_pts(x, y, w, h, r);
        self.fill(&p, c);
        if lw > 0.0 {
            self.stroke(&p, lw, INK, true);
        }
    }
    fn rect(&self, x: f32, y: f32, w: f32, h: f32, c: Color) {
        self.fill_c(&[vec2(x, y), vec2(x + w, y), vec2(x + w, y + h), vec2(x, y + h)], self.kc(c));
    }
    fn ln(&self, x0: f32, y0: f32, x1: f32, y1: f32, lw: f32, c: &str) {
        self.stroke(&[vec2(x0, y0), vec2(x1, y1)], lw, c, false);
    }
    fn star(&self, x: f32, y: f32, r: f32, rot: f32, c: &str) {
        let (s, co) = rot.sin_cos();
        let p: Vec<Vec2> = star_pts(r).iter().map(|v| vec2(x + v.x * co - v.y * s, y + v.x * s + v.y * co)).collect();
        self.fill(&p, c);
    }
    // align: 1 centro, 0 izquierda. ow: contorno blanco.
    fn txt(&self, s: &str, x: f32, y: f32, size: f32, c: &str, align: i32, ow: f32) {
        let sc = self.lscale();
        let fs = (size * sc * self.px).round().max(6.0) as u16;
        let fscale = 1.0 / self.px;
        let d = measure_text(s, Some(&self.font), fs, fscale);
        let w_local = d.width / sc;
        let lx = if align == 1 { x - w_local / 2.0 } else { x };
        let ly = y + d.offset_y / sc * 0.5 - 2.0;
        let rot = self.m.matrix2.x_axis.y.atan2(self.m.matrix2.x_axis.x);
        let base = self.tp(vec2(lx, ly));
        let mut p = TextParams { font: Some(&self.font), font_size: fs, font_scale: fscale, rotation: rot, color: self.k("#ffffff"), ..Default::default() };
        if ow > 0.0 {
            let o = ow * 0.55 * sc;
            for i in 0..8 {
                let a = i as f32 * TAU / 8.0;
                draw_text_ex(s, base.x + a.cos() * o, base.y + a.sin() * o, p.clone());
            }
        }
        p.color = self.k(c);
        draw_text_ex(s, base.x, base.y, p);
    }
}

// ---------- entidades ----------
#[derive(Clone, Copy, PartialEq)]
enum PT { G, B, K, W }
#[derive(Clone, Copy, PartialEq, Default)]
enum Item { #[default] None, Spring, Tramp, Prop, Jet }

#[derive(Clone)]
struct Plat { x: f32, y: f32, w: f32, t: PT, vx: f32, item: Item, ix: f32, used: f32, broken: bool, vy: f32, gone: bool, rot: f32, x0: f32, rng: f32 }
#[derive(Clone)]
struct Mon { x: f32, y: f32, bx: f32, bat: bool, w: f32, h: f32, ph: f32, dead: bool, vy: f32 }
struct Coin { x: f32, y: f32, ph: f32, got: bool }
struct Bullet { x: f32, y: f32 }
struct Part { x: f32, y: f32, vx: f32, vy: f32, life: f32, max: f32, size: f32, col: Color, star: bool, rot: f32, grav: f32 }
struct Ring { x: f32, y: f32, t: f32, col: Color }
struct FTxt { x: f32, y: f32, t: f32, s: String }
struct Chain { y: f32, x: f32, t: PT, l0: f32, l1: f32 }

#[derive(Default)]
struct Player { x: f32, y: f32, vx: f32, vy: f32, face: f32, pu: Item, put: f32, shoot_t: f32, sq: f32, flip: f32, dead: bool }

#[derive(PartialEq, Clone, Copy)]
enum State { Menu, Play, Pause, Over }

fn pc(t: PT) -> (&'static str, &'static str) {
    match t {
        PT::G => ("#86d443", "#5aa82a"),
        PT::B => ("#6cc3ee", "#3c93c4"),
        PT::K => ("#b8865a", "#8a5e3a"),
        PT::W => ("#ffffff", "#dddddd"),
    }
}

// ---------- guardado ----------
fn save_path() -> std::path::PathBuf {
    #[cfg(target_os = "android")]
    {
        let d = std::path::PathBuf::from("/data/data/com.nyxusss.bubujump/files");
        let _ = std::fs::create_dir_all(&d);
        return d.join("save.txt");
    }
    #[allow(unreachable_code)]
    {
        let base = std::env::var("HOME").map(std::path::PathBuf::from).unwrap_or_else(|_| std::path::PathBuf::from("."));
        base.join(".bubujump.sav")
    }
}

struct Save { coins: i64, dudu: bool, mode: usize, dif: usize, bests: Vec<(String, i64)> }
impl Save {
    fn load() -> Save {
        let mut s = Save { coins: 0, dudu: false, mode: 0, dif: 1, bests: vec![] };
        if let Ok(txt) = std::fs::read_to_string(save_path()) {
            for l in txt.lines() {
                if let Some((k, v)) = l.split_once('=') {
                    let n: i64 = v.trim().parse().unwrap_or(0);
                    match k {
                        "coins" => s.coins = n,
                        "dudu" => s.dudu = n == 1,
                        "mode" => s.mode = (n as usize).min(MODES.len() - 1),
                        "dif" => s.dif = (n as usize).min(2),
                        _ if k.starts_with("best") => s.bests.push((k.to_string(), n)),
                        _ => {}
                    }
                }
            }
        }
        s
    }
    fn write(&self) {
        let mut o = format!("coins={}\ndudu={}\nmode={}\ndif={}\n", self.coins, self.dudu as i32, self.mode, self.dif);
        for (k, v) in &self.bests {
            o += &format!("{}={}\n", k, v);
        }
        let _ = std::fs::write(save_path(), o);
    }
    fn best_key(&self) -> String {
        match (self.mode, self.dif) {
            (0, 1) => "best".into(),
            (1, d) => format!("best_t60_{}", d),
            (m, d) => format!("best_{}{}", MODES[m].0, d),
        }
    }
    fn best(&self) -> i64 {
        let k = self.best_key();
        self.bests.iter().find(|(a, _)| *a == k).map(|(_, v)| *v).unwrap_or(0)
    }
    fn set_best(&mut self, v: i64) {
        let k = self.best_key();
        if let Some(e) = self.bests.iter_mut().find(|(a, _)| *a == k) {
            e.1 = v;
        } else {
            self.bests.push((k, v));
        }
    }
}

struct Btn { x: f32, y: f32, w: f32, h: f32, id: u8 }

struct Game {
    w: f32,
    h: f32,
    state: State,
    p: Player,
    plats: Vec<Plat>,
    mons: Vec<Mon>,
    coins: Vec<Coin>,
    bullets: Vec<Bullet>,
    parts: Vec<Part>,
    rings: Vec<Ring>,
    ftxt: Vec<FTxt>,
    chains: Vec<Chain>,
    safe_tab: Vec<f32>,
    score: f32,
    cam_off: f32,
    shake: f32,
    flash: f32,
    time: f32,
    over_t: f32,
    tl: f32,
    combo: i32,
    combo_y: f32,
    combo_pop: f32,
    run_max_combo: i32,
    run_coins: i64,
    new_best: bool,
    fire_t: f32,
    swap_t: f32,
    sv: Save,
    btns: Vec<Btn>,
    touch_dir: f32,
    touch_fire: bool,
    touch_ui: bool,
    zones: [bool; 3],
}

impl Game {
    fn new() -> Game {
        let mut g = Game {
            w: BASE_W, h: BASE_H, state: State::Menu, p: Player::default(), plats: vec![], mons: vec![], coins: vec![], bullets: vec![],
            parts: vec![], rings: vec![], ftxt: vec![], chains: vec![], safe_tab: vec![], score: 0.0, cam_off: 0.0, shake: 0.0, flash: 0.0,
            time: 0.0, over_t: 0.0, tl: TIME_LIMIT, combo: 0, combo_y: 1e9, combo_pop: 0.0, run_max_combo: 0, run_coins: 0, new_best: false,
            fire_t: 0.0, swap_t: 0.0, sv: Save::load(), btns: vec![], touch_dir: 0.0, touch_fire: false,
            touch_ui: cfg!(target_os = "android"), zones: [false; 3],
        };
        g.build_safe_tab();
        g
    }
    fn dif(&self) -> (f32, f32, f32) {
        let d = DIFS[self.sv.dif];
        (d.1, d.2, d.3)
    }
    fn zen(&self) -> bool {
        MODES[self.sv.mode].0 == "zen"
    }
    fn timed(&self) -> bool {
        MODES[self.sv.mode].0 == "time"
    }

    // ----- alcance de salto (idéntico a game.gd) -----
    fn air_time(dh: f32) -> f32 {
        let d = JUMP * JUMP - 2.0 * GRAV * dh;
        if d < 0.0 { -1.0 } else { (-JUMP + d.sqrt()) / GRAV }
    }
    fn x_reach(dh: f32, vt: f32, k: f32) -> f32 {
        let t = Self::air_time(dh);
        if t < 0.0 { 0.0 } else { vt * t - vt * (1.0 - (-k * t).exp()) / k }
    }
    fn build_safe_tab(&mut self) {
        self.safe_tab = (0..(REACH as usize + 2)).map(|i| {
            let dh = i as f32;
            let r = Self::x_reach(dh, SPEED, 10.0).min(Self::x_reach(dh, SPEED, SNOW_K).min(Self::x_reach(dh, SPEED - WIND_V, 10.0)));
            42.0 + MARGIN * r
        }).collect();
    }
    fn safe_dx(&self, dv: f32) -> f32 {
        self.safe_tab[(dv.ceil() as i32).clamp(0, self.safe_tab.len() as i32 - 1) as usize]
    }
    fn wdist(&self, a: f32, b: f32) -> f32 {
        let d = (a - b).abs();
        d.min(self.w + 44.0 - d)
    }

    fn mk(&self, x: f32, y: f32, t: PT) -> Plat {
        let vx = if t == PT::B { (if chance() < 0.5 { -1.0 } else { 1.0 }) * (50.0 + (self.score / 80.0).min(90.0)) } else { 0.0 };
        Plat { x, y, w: 58.0, t, vx, item: Item::None, ix: 0.0, used: 0.0, broken: false, vy: 0.0, gone: false, rot: 0.0, x0: x, rng: 45.0 }
    }

    fn free_spot(&self, x: f32, y: f32, w: f32, mov: bool) -> bool {
        let hx_ = if self.chains.len() <= 1 { 1e5 } else { 170.0 };
        for p in &self.plats {
            if p.broken || p.gone || (p.x - x).abs() > hx_ {
                continue;
            }
            let dy = (p.y - y).abs();
            if dy < 38.0 || ((mov || p.t == PT::B) && dy < 50.0) {
                return false;
            }
        }
        for m in &self.mons {
            if m.dead {
                continue;
            }
            let sw = if m.bat { 40.0 } else { 14.0 } + m.w / 2.0 + 10.0;
            if (m.y - y).abs() < m.h / 2.0 + 26.0 && x < m.x + sw && x + w > m.x - sw {
                return false;
            }
        }
        for c in &self.coins {
            if (c.y - y).abs() < 20.0 && c.x > x - 10.0 && c.x < x + w + 10.0 {
                return false;
            }
        }
        true
    }

    fn setup_chains(&mut self) {
        let n = if self.w > 950.0 { 3 } else if self.w > 600.0 { 2 } else { 1 };
        let lw = self.w / n as f32;
        let start = (((self.w / 2.0) / lw) as i32).clamp(0, n - 1);
        self.chains = (0..n).map(|i| {
            let mut c = Chain { y: self.h - 60.0, x: i as f32 * lw + lw / 2.0 - 29.0, t: PT::G, l0: i as f32 * lw, l1: (i + 1) as f32 * lw };
            if i == start {
                c.x = self.w / 2.0 - 29.0;
            } else {
                c.y = self.h - 60.0 - rnd(30.0, 90.0);
            }
            c
        }).collect();
    }

    fn gen(&mut self) {
        for i in 0..self.chains.len() {
            while self.chains[i].y > -60.0 {
                self.gen_one(i);
            }
        }
    }

    fn gen_one(&mut self, ci: usize) {
        let (dg, dm, dc) = self.dif();
        let (cy0, cx0, ct, l0, l1) = { let c = &self.chains[ci]; (c.y, c.x, c.t, c.l0, c.l1) };
        let mxg = REACH.min((90.0 + self.score / 70.0) * dg);
        let gap = rnd(mxg * 0.65, if ct == PT::B { mxg.min(118.0) } else { mxg });
        let pb = (self.score / 14000.0).min(0.35);
        let pw = if self.score > 3000.0 { (self.score / 30000.0).min(0.2) } else { 0.0 };
        let r = chance();
        let mut t = if r < pb { PT::B } else if r < pb + pw { PT::W } else { PT::G };
        if ct == PT::B {
            t = PT::G;
        }
        let y0 = cy0 - gap;
        let slack = if t == PT::B { B_SLACK } else { 0.0 } + if ct == PT::B { B_SLACK } else { 0.0 };
        let (mut found, mut y, mut x) = (false, 0.0, 0.0);
        let xlo = 4f32.max(l0);
        let xhi = (self.w - 62.0).min(l1 - 58.0).max(xlo + 1.0);
        for k in 0..40 {
            if found {
                break;
            }
            let cy = y0 + if k % 2 == 1 { 1.0 } else { -1.0 } * ((k as f32) / 2.0).ceil() * 6.0;
            let dv = cy0 - cy;
            if dv > REACH || dv < 40.0 {
                continue;
            }
            let lim = self.safe_dx(dv) - slack;
            let mut bs = -1.0;
            for _ in 0..12 {
                let cx = rnd(xlo, xhi);
                if self.wdist(cx, cx0) > lim || !self.free_spot(cx, cy, 58.0, t == PT::B) {
                    continue;
                }
                let mut md: f32 = 100.0;
                for p in &self.plats {
                    if !p.broken && !p.gone && (p.y - cy).abs() < 130.0 {
                        md = md.min((p.x - cx).abs());
                    }
                }
                let s = md + rnd(0.0, 35.0);
                if s > bs {
                    bs = s;
                    x = cx;
                    y = cy;
                    found = true;
                }
            }
        }
        if !found {
            y = cy0 - 70.0;
            x = cx0;
            if t == PT::B {
                t = PT::G;
            }
            self.plats.retain(|q| !(q.t == PT::K && (q.y - y).abs() < 40.0 && (q.x - x).abs() < 170.0));
            self.mons.retain(|m| (m.y - y).abs() >= 50.0 || (m.x - x).abs() > 170.0);
        }
        let prev_y = cy0;
        {
            let c = &mut self.chains[ci];
            c.y = y;
            c.x = x;
            c.t = t;
        }
        let mut p = self.mk(x, y, t);
        if t != PT::W {
            let q = chance();
            p.item = if q < 0.06 { Item::Spring } else if q < 0.08 && self.score > 1000.0 { Item::Tramp } else if q < 0.095 && self.score > 700.0 { Item::Prop } else if q < 0.105 && self.score > 2500.0 { Item::Jet } else { Item::None };
            p.ix = if p.item == Item::Tramp { 10.0 } else { rnd(8.0, 38.0) };
        }
        let item = p.item;
        self.plats.push(p);

        if self.score > 300.0 && chance() < 0.22 {
            let ky = (y + prev_y) / 2.0;
            for _ in 0..5 {
                let kx = rnd(xlo, xhi);
                if self.free_spot(kx, ky, 58.0, false) {
                    let k = self.mk(kx, ky, PT::K);
                    self.plats.push(k);
                    break;
                }
            }
        }
        if !self.zen() && self.score > 1500.0 && chance() < (self.score / 60000.0).min(0.08) * dm && item == Item::None {
            let big = chance() < 0.5;
            let (w, h) = if big { (62.0, 40.0) } else { (56.0, 34.0) };
            let my = y - 45.0;
            let mxp = rnd(40f32.max(l0), (self.w - 40.0).min(l1).max(41f32.max(l0) + 1.0));
            let sw = if big { 14.0 } else { 40.0 } + w / 2.0 + 10.0;
            let ok = !self.plats.iter().any(|q| !q.broken && (q.y - my).abs() < h / 2.0 + 26.0 && q.x < mxp + sw && q.x + q.w > mxp - sw)
                && !self.mons.iter().any(|m| (m.y - my).abs() < 90.0 && (m.x - mxp).abs() < 170.0);
            if ok {
                self.mons.push(Mon { x: mxp, y: my, bx: 0.0, bat: !big, w, h, ph: rnd(0.0, 6.0), dead: false, vy: 0.0 });
            }
        }
        if item == Item::None && t != PT::B && chance() < 0.4 * dc {
            let n = if chance() < 0.25 { 3 } else { 1 };
            let (cy2, cx2) = (y - 34.0, x + 29.0);
            if !self.mons.iter().any(|m| !m.dead && (m.y - cy2).abs() < 50.0 && (m.x - cx2).abs() < 90.0) {
                for i in 0..n {
                    self.coins.push(Coin { x: cx2 + (i as f32 - (n - 1) as f32 / 2.0) * 20.0, y: cy2, ph: rnd(0.0, 6.0), got: false });
                }
            }
        }
    }

    fn reset(&mut self) {
        self.p = Player { x: self.w / 2.0, y: self.h - 60.0, vy: JUMP, face: 1.0, ..Default::default() };
        self.plats.clear();
        self.mons.clear();
        self.bullets.clear();
        self.parts.clear();
        self.rings.clear();
        self.ftxt.clear();
        self.coins.clear();
        self.score = 0.0;
        self.cam_off = 0.0;
        self.shake = 0.0;
        self.flash = 0.0;
        self.new_best = false;
        self.run_coins = 0;
        self.tl = TIME_LIMIT;
        self.combo = 0;
        self.combo_y = 1e9;
        self.combo_pop = 0.0;
        self.run_max_combo = 0;
        let p0 = self.mk(self.w / 2.0 - 29.0, self.h - 60.0, PT::G);
        self.plats.push(p0);
        self.setup_chains();
        self.gen();
    }
    fn start(&mut self) {
        self.reset();
        self.state = State::Play;
    }

    // ----- efectos -----
    #[allow(clippy::too_many_arguments)]
    fn fx(&mut self, x: f32, y: f32, n: usize, cols: &[&str], sp: f32, grav: f32, life: f32, size: f32, star: bool, up: f32) {
        for _ in 0..n {
            let a = rnd(0.0, TAU);
            let s = rnd(sp * 0.4, sp);
            let l = life * rnd(0.7, 1.1);
            self.parts.push(Part {
                x, y, vx: a.cos() * s, vy: a.sin() * s + up, life: l, max: l,
                size: (if star { 6.0 } else { 3.5 }) * size * rnd(0.7, 1.2), col: hx(pick(cols)), star, rot: rnd(0.0, TAU), grav,
            });
        }
    }
    fn ring(&mut self, x: f32, y: f32, c: &str) {
        self.rings.push(Ring { x, y, t: 0.0, col: hx(c) });
    }
    fn float_txt(&mut self, x: f32, y: f32, s: &str) {
        self.ftxt.push(FTxt { x, y, t: 0.0, s: s.to_string() });
    }
    fn update_fx(&mut self, dt: f32) {
        for p in &mut self.parts {
            p.vy += p.grav * dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life -= dt;
            p.rot += dt * 3.0;
        }
        self.parts.retain(|p| p.life > 0.0);
        if self.parts.len() > 400 {
            let k = self.parts.len() - 400;
            self.parts.drain(0..k);
        }
        for r in &mut self.rings {
            r.t += dt * 2.2;
        }
        self.rings.retain(|r| r.t < 1.0);
        for f in &mut self.ftxt {
            f.t += dt;
            f.y -= 40.0 * dt;
        }
        self.ftxt.retain(|f| f.t < 1.0);
    }

    fn combo_mult(&self) -> f32 {
        if self.combo < 3 { 1.0 } else { 1.0 + (self.combo.min(40) as f32) * 0.05 }
    }
    fn combo_land(&mut self, wy: f32) {
        if wy < self.combo_y - 4.0 {
            self.combo += 1;
            self.combo_pop = 1.0;
            if self.combo >= 3 && self.combo % 5 == 0 {
                let (x, y) = (self.p.x, self.p.y - 70.0);
                self.float_txt(x, y, &format!("combo x{}", self.combo));
            }
        } else {
            self.combo = 1;
        }
        self.combo_y = wy;
        self.run_max_combo = self.run_max_combo.max(self.combo);
    }
    fn combo_break(&mut self) {
        if self.combo >= 5 && self.state == State::Play && !self.p.dead {
            let (x, y) = (self.p.x, self.p.y - 70.0);
            self.float_txt(x, y, "combo perdido");
        }
        self.combo = 0;
        self.combo_y = 1e9;
    }
    fn alt_m(&self) -> i64 {
        (self.cam_off / PX_PER_M).floor() as i64
    }

    fn shoot(&mut self) {
        if self.p.dead || self.p.pu != Item::None {
            return;
        }
        self.bullets.push(Bullet { x: self.p.x, y: self.p.y - 34.0 });
        self.p.shoot_t = 0.25;
        let (x, y) = (self.p.x, self.p.y - 40.0);
        self.fx(x, y, 3, &["#ffffff", "#f3d23c"], 50.0, 0.0, 0.3, 1.0, false, 0.0);
    }
    fn kill_mon(&mut self, i: usize) {
        self.mons[i].dead = true;
        self.score += 50.0;
        self.shake = 6.0;
        let (x, y) = (self.mons[i].x + self.mons[i].bx, self.mons[i].y);
        self.fx(x, y, 12, &STARC, 220.0, 500.0, 0.9, 1.0, true, 0.0);
        self.ring(x, y, "#f3d23c");
        self.float_txt(x, y - 30.0, "+50");
    }
    fn hurt(&mut self) {
        if self.p.dead {
            return;
        }
        self.p.dead = true;
        self.p.vy = self.p.vy.max(150.0);
        self.combo_break();
        self.shake = 12.0;
        self.flash = 1.0;
        let (x, y) = (self.p.x, self.p.y - 30.0);
        self.fx(x, y, 14, &["#f3d23c", "#ffffff", "#e8504a"], 200.0, 300.0, 0.8, 1.0, true, 0.0);
    }
    fn game_over(&mut self) {
        self.state = State::Over;
        self.over_t = 0.0;
        self.shake = 8.0;
        self.combo_break();
        if self.score as i64 > self.sv.best() {
            self.sv.set_best(self.score as i64);
            self.new_best = true;
        }
        self.sv.write();
    }

    fn input_dir(&self) -> f32 {
        let mut d = 0.0;
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            d -= 1.0;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            d += 1.0;
        }
        (d + self.touch_dir).clamp(-1.0, 1.0)
    }

    fn step(&mut self, dt: f32) {
        if self.timed() {
            self.tl -= dt;
            if self.tl <= 0.0 {
                self.tl = 0.0;
                self.game_over();
                return;
            }
        }
        let dir = self.input_dir();
        if self.touch_fire {
            self.fire_t -= dt;
            if self.fire_t <= 0.0 {
                self.shoot();
                self.fire_t = 0.3;
            }
        }
        let target = dir * SPEED;
        self.p.vx += (target - self.p.vx) * (dt * 10.0).min(1.0);
        if dir.abs() > 0.1 {
            self.p.face = dir.signum();
        }
        self.p.x += self.p.vx * dt;
        if self.p.x < -22.0 {
            self.p.x = self.w + 22.0;
        }
        if self.p.x > self.w + 22.0 {
            self.p.x = -22.0;
        }
        let prev_y = self.p.y;
        if self.p.pu != Item::None {
            self.p.put -= dt;
            self.p.vy = if self.p.pu == Item::Jet { (self.p.vy - 2400.0 * dt).max(-980.0) } else { -640.0 };
            if self.p.put <= 0.0 {
                self.p.pu = Item::None;
            }
        } else {
            self.p.vy += GRAV * dt;
        }
        self.p.y += self.p.vy * dt;
        self.p.sq = (self.p.sq - dt * 4.0).max(0.0);
        self.p.shoot_t = (self.p.shoot_t - dt).max(0.0);
        if self.p.flip > 0.0 {
            self.p.flip = (self.p.flip - dt * 1.6).max(0.0);
        }
        if self.p.pu == Item::Jet {
            for _ in 0..2 {
                let (x, y) = (self.p.x - self.p.face * 16.0 + rnd(-5.0, 5.0), self.p.y - 6.0);
                let c = hx(pick(&["#f39b2d", "#e8504a", "#f3d23c"]));
                let l = rnd(0.25, 0.45);
                self.parts.push(Part { x, y, vx: rnd(-20.0, 20.0), vy: rnd(150.0, 260.0), life: l, max: l, size: rnd(3.0, 6.0), col: c, star: false, rot: 0.0, grav: 0.0 });
            }
        } else if self.p.pu == Item::Prop && chance() < 0.5 {
            let (x, y) = (self.p.x + rnd(-14.0, 14.0), self.p.y - 66.0);
            self.fx(x, y, 1, &["#f3d23c", "#ffffff"], 30.0, 120.0, 0.6, 0.7, true, 0.0);
        }
        let w = self.w;
        for p in &mut self.plats {
            if p.t == PT::B && !p.broken {
                p.x += p.vx * dt;
                let lo = 2f32.max(p.x0 - p.rng);
                let hi = (w - p.w - 2.0).min(p.x0 + p.rng);
                if p.x < lo || p.x > hi {
                    p.x = p.x.clamp(lo, hi);
                    p.vx *= -1.0;
                }
            }
            if p.broken {
                p.vy += GRAV * dt;
                p.y += p.vy * dt;
                p.rot += dt * 2.0;
            }
            if p.used > 0.0 {
                p.used = (p.used - dt * 3.0).max(0.0);
            }
        }
        if !self.p.dead {
            // potenciadores
            if self.p.pu == Item::None {
                for i in 0..self.plats.len() {
                    let it = self.plats[i].item;
                    if it == Item::Prop || it == Item::Jet {
                        let ix = self.plats[i].x + self.plats[i].ix + 6.0;
                        let iy = self.plats[i].y - 10.0;
                        if (self.p.x - ix).abs() < 22.0 && self.p.y > iy - 6.0 && self.p.y - 44.0 < iy + 6.0 {
                            self.p.pu = it;
                            self.p.put = if it == Item::Jet { 2.6 } else { 3.2 };
                            self.plats[i].item = Item::None;
                            self.p.flip = 0.0;
                            let (x, y) = (self.p.x, self.p.y - 30.0);
                            self.ring(x, y, "#6cc3ee");
                            self.fx(x, y, 10, &STARC, 160.0, 200.0, 0.8, 1.0, true, 0.0);
                            break;
                        }
                    }
                }
            }
            // aterrizaje
            if self.p.vy > 0.0 && self.p.pu == Item::None {
                for i in 0..self.plats.len() {
                    let p = self.plats[i].clone();
                    if p.broken || p.gone {
                        continue;
                    }
                    if prev_y <= p.y + 2.0 && self.p.y >= p.y && self.p.x + 13.0 > p.x && self.p.x - 13.0 < p.x + p.w {
                        self.combo_land(p.y - self.cam_off);
                        if p.t == PT::K {
                            self.p.y = p.y;
                            self.p.sq = 0.6;
                            self.p.vy = JUMP * 0.5;
                            self.plats[i].broken = true;
                            continue;
                        }
                        self.p.y = p.y;
                        self.p.sq = 1.0;
                        let ix = p.x + p.ix;
                        let (px, py) = (self.p.x, self.p.y);
                        if p.item == Item::Spring && px + 13.0 > ix && px - 13.0 < ix + 14.0 {
                            self.p.vy = SPRING;
                            self.plats[i].used = 1.0;
                            self.shake = 3.0;
                            self.fx(px, py, 8, &STARC, 150.0, 300.0, 0.7, 1.0, true, 0.0);
                        } else if p.item == Item::Tramp && px + 13.0 > ix && px - 13.0 < ix + 38.0 {
                            self.p.vy = TRAMP;
                            self.plats[i].used = 1.0;
                            self.p.flip = 1.0;
                            self.shake = 4.0;
                            self.ring(px, py, "#e8504a");
                            self.fx(px, py, 14, &STARC, 220.0, 300.0, 0.9, 1.0, true, 0.0);
                        } else {
                            self.p.vy = JUMP;
                            self.fx(px, py, 6, &["#ffffff", "#f3e6d4", pc(p.t).0], 70.0, -40.0, 0.6, 1.3, false, -10.0);
                        }
                        if p.t == PT::W {
                            self.plats[i].gone = true;
                            self.fx(p.x + 29.0, p.y + 7.0, 12, &["#ffffff", "#eeeeee"], 90.0, -30.0, 0.7, 1.8, false, 0.0);
                        }
                        break;
                    }
                }
            }
            // monstruos
            for i in 0..self.mons.len() {
                if self.mons[i].dead || self.p.dead {
                    continue;
                }
                let m = self.mons[i].clone();
                let mx = m.x + m.bx;
                if (self.p.x - mx).abs() < m.w / 2.0 + 8.0 && self.p.y > m.y - m.h / 2.0 && self.p.y - 44.0 < m.y + m.h / 2.0 {
                    if self.p.pu != Item::None {
                        self.kill_mon(i);
                        continue;
                    }
                    if self.p.vy > 0.0 && prev_y <= m.y - m.h / 2.0 + 10.0 {
                        self.kill_mon(i);
                        self.p.vy = JUMP;
                        self.p.sq = 1.0;
                        self.combo += 1;
                        self.combo_pop = 1.0;
                    } else {
                        self.hurt();
                    }
                }
            }
        }
        for m in &mut self.mons {
            m.ph += dt * 3.0;
            m.bx = (m.ph * 0.7).sin() * if m.bat { 40.0 } else { 14.0 };
            if m.dead {
                m.vy += GRAV * dt;
                m.y += m.vy * dt;
            }
        }
        // monedas
        let mut got = vec![];
        for c in &mut self.coins {
            if !c.got && !self.p.dead && (self.p.x - c.x).abs() < 24.0 && c.y > self.p.y - 54.0 && c.y < self.p.y + 6.0 {
                c.got = true;
                got.push((c.x, c.y));
            }
        }
        for (x, y) in got {
            self.sv.coins += 1;
            self.run_coins += 1;
            self.fx(x, y, 5, &["#f3d23c", "#ffffff"], 90.0, 100.0, 0.5, 0.6, true, 0.0);
        }
        // balas
        for bi in 0..self.bullets.len() {
            self.bullets[bi].y -= 900.0 * dt;
            for mi in 0..self.mons.len() {
                let (b, m) = (&self.bullets[bi], &self.mons[mi]);
                if !m.dead && (b.x - (m.x + m.bx)).abs() < m.w / 2.0 && (b.y - m.y).abs() < m.h / 2.0 {
                    self.kill_mon(mi);
                    self.bullets[bi].y = -99.0;
                }
            }
        }
        self.bullets.retain(|b| b.y > -20.0);
        // cámara
        if !self.p.dead && self.p.y < self.h * 0.42 {
            let d = self.h * 0.42 - self.p.y;
            self.p.y += d;
            self.cam_off += d;
            self.score += d * 0.6 * self.combo_mult();
            for c in &mut self.chains { c.y += d; }
            for q in &mut self.plats { q.y += d; }
            for q in &mut self.mons { q.y += d; }
            for q in &mut self.bullets { q.y += d; }
            for q in &mut self.parts { q.y += d; }
            for q in &mut self.coins { q.y += d; }
            for q in &mut self.rings { q.y += d; }
            for q in &mut self.ftxt { q.y += d; }
        }
        let h = self.h;
        self.plats.retain(|p| p.y < h + 40.0 && !p.gone);
        self.mons.retain(|m| m.y < h + 80.0);
        self.coins.retain(|c| !c.got && c.y < h + 40.0);
        self.gen();
        if self.combo > 0 && self.p.pu == Item::None && (self.p.y - self.cam_off) > self.combo_y + 160.0 {
            self.combo_break();
        }
        self.combo_pop = (self.combo_pop - dt * 4.0).max(0.0);
        if self.p.y - 50.0 > self.h {
            self.game_over();
        }
    }

    fn night_k(&self) -> f32 {
        if matches!(self.state, State::Play | State::Pause | State::Over) {
            smoothstep((DUSK_M + 150.0) * PX_PER_M, NIGHT_M * PX_PER_M, self.cam_off)
        } else {
            0.0
        }
    }
    fn dusk_k(&self) -> f32 {
        if matches!(self.state, State::Play | State::Pause | State::Over) {
            smoothstep(DUSK_M * 0.6 * PX_PER_M, DUSK_M * PX_PER_M, self.cam_off)
        } else {
            0.0
        }
    }

    // ---------- dibujo ----------
    fn paper(&self, g: &mut Painter) {
        let (nk, dk) = (self.night_k(), self.dusk_k());
        let bg = lerpc(lerpc(hx("#f8f0e3"), hx("#f8d3b4"), dk), hx("#1c1f3d"), nk);
        g.rect(-20.0, -20.0, self.w + 40.0, self.h + 40.0, bg);
        let grid = lerpc(lerpc(hx("#ecd9c2"), Color::new(0.93, 0.78, 0.7, 1.0), dk), Color::new(0.2, 0.24, 0.42, 1.0), nk);
        let mut x = 0.0;
        while x <= self.w {
            g.rect(x + 0.1, -20.0, 0.8, self.h + 40.0, grid);
            x += 16.0;
        }
        let mut y = -16.0 + self.cam_off % 16.0;
        while y <= self.h + 16.0 {
            g.rect(-20.0, y + 0.1, self.w + 40.0, 0.8, grid);
            y += 16.0;
        }
        if nk > 0.02 {
            let r = 26.0 * (0.8 + nk * 0.2);
            let (mx, my) = (self.w - 70.0, 120.0);
            g.save();
            g.ga *= nk;
            g.fcirc(mx, my, r, "#fff3c4", 2.0);
            g.fcirc(mx - r * 0.3, my - r * 0.2, r * 0.18, "#eadcaa", 0.0);
            g.fcirc(mx + r * 0.3, my + r * 0.25, r * 0.12, "#eadcaa", 0.0);
            g.restore();
        }
        if nk > 0.15 {
            let sa = ((nk - 0.15) * 2.0).min(1.0);
            let n = 14 + (nk * 26.0) as i32;
            for i in 0..n {
                let sy = fposmod(i as f32 * 211.0 + self.cam_off * (0.05 + (i % 3) as f32 * 0.02), self.h + 40.0) - 20.0;
                let sx = (i as f32 * 131.0 + (i / 14) as f32 * 67.0) % self.w;
                let tw = 0.55 + 0.45 * (self.time * (1.5 + (i % 5) as f32 * 0.6) + i as f32 * 1.7).sin();
                g.save();
                g.ga *= sa * (0.7 + (tw - 0.7) * nk);
                if i % 4 == 0 || nk < 0.3 {
                    g.star(sx, sy, 3.0 + (i % 3) as f32, self.time * 0.5 + i as f32, if i % 3 != 0 || nk < 0.3 { "#f3d23c" } else { "#ffffff" });
                } else {
                    g.fcirc(sx, sy, 1.2 + (i % 3) as f32 * 0.6, "#fff8e0", 0.0);
                }
                g.restore();
            }
        }
        if nk > 0.05 && self.state != State::Menu {
            for j in 0..6 {
                g.save();
                g.ga *= nk * 0.045;
                g.fcirc(self.p.x, self.p.y - 30.0, 100.0 - j as f32 * 14.0, "#ffe9a8", 0.0);
                g.restore();
            }
        }
        let cp = cloud_pts();
        let ccol = lerpc(hx("#fffaf2"), hx("#3b4170"), nk);
        let cline = lerpc(hx("#cdb8a0"), hx("#5a6094"), nk);
        for i in 0..6 {
            let cy = fposmod(i as f32 * 173.0 + self.cam_off * 0.25 + self.time * 4.0, self.h + 160.0) - 80.0;
            let cx = fposmod(i as f32 * 97.0 + self.time * (6.0 + i as f32 * 2.0), self.w + 120.0) - 60.0;
            let s = 0.8 + (i % 3) as f32 * 0.3;
            g.save();
            g.ga *= 0.55;
            g.tr(cx, cy);
            g.sc(s, s);
            g.fill_c(&cp, g.kc(ccol));
            g.stroke_c(&cp, 1.8, g.kc(cline), true);
            g.restore();
        }
    }

    fn plat(&self, g: &mut Painter, p: &Plat) {
        let (c, d) = pc(p.t);
        if p.broken {
            for s in [-1.0f32, 1.0] {
                g.save();
                g.tr(p.x + p.w / 2.0 + s * (8.0 + p.vy * 0.02), p.y + 7.0);
                g.rot(s * p.rot);
                g.fr(-p.w / 4.0 - 2.0, -7.0, p.w / 2.0, 14.0, 6.0, c, 2.0);
                g.restore();
            }
            return;
        }
        g.save();
        g.tr(p.x, p.y);
        g.fill(&rr_pts(3.0, 5.0, 58.0, 14.0, 7.0), "#3b242014");
        let body = rr_pts(0.0, 0.0, 58.0, 14.0, 7.0);
        g.fill(&body, c);
        g.fill(&rr_pts(3.0, 9.0, 52.0, 3.0, 2.0), d);
        g.stroke(&body, 2.0, INK, true);
        g.ln(8.0, 4.5, 18.0, 4.5, 1.4, "#ffffff99");
        g.ln(38.0, 5.0, 44.0, 5.0, 1.4, "#ffffff99");
        if p.t == PT::K {
            g.stroke(&[vec2(29.0, 0.0), vec2(26.0, 7.0), vec2(31.0, 14.0)], 1.5, INK, false);
        }
        g.restore();
        if p.item != Item::None {
            self.item(g, p);
        }
    }

    fn item(&self, g: &mut Painter, p: &Plat) {
        let (x, y) = (p.x + p.ix, p.y);
        match p.item {
            Item::Spring => {
                let h = if p.used > 0.0 { 6.0 } else { 12.0 };
                let pts: Vec<Vec2> = (0..5).map(|i| vec2(x + if i % 2 == 1 { 12.0 } else { 2.0 }, y - i as f32 * h / 4.0)).collect();
                g.stroke(&pts, 2.0, INK, false);
                g.fr(x, y - h - 3.0, 14.0, 4.0, 2.0, "#cccccc", 1.6);
            }
            Item::Tramp => {
                let s = if p.used > 0.0 { 3.0 } else { 0.0 };
                g.ln(x + 4.0, y, x + 8.0, y - 7.0, 2.0, INK);
                g.ln(x + 34.0, y, x + 30.0, y - 7.0, 2.0, INK);
                g.fe(x + 19.0, y - 8.0 + s, 19.0, 5.0 - s * 0.5, "#e8504a", 2.0);
                g.fe(x + 19.0, y - 9.0 + s, 11.0, 2.5, "#ff66aa", 0.0);
            }
            Item::Prop | Item::Jet => {
                let bob = (self.time * 4.0 + p.x).sin() * 2.0;
                g.save();
                g.ga *= 0.35 + (self.time * 6.0).sin() * 0.15;
                g.fcirc(x + 6.0, y - 12.0 + bob, 16.0, "#fff3a0", 0.0);
                g.restore();
                g.save();
                g.tr(0.0, bob);
                if p.item == Item::Prop {
                    self.hat(g, x + 6.0, y - 2.0, 0.0);
                } else {
                    self.jet(g, x + 6.0, y - 24.0, false);
                }
                g.restore();
            }
            Item::None => {}
        }
    }
    fn hat(&self, g: &mut Painter, x: f32, y: f32, t: f32) {
        let dome = arc_pts(x, y, 10.0, PI, 0.0, 10);
        g.fill(&dome, "#e8504a");
        g.stroke(&dome, 2.0, INK, true);
        let mut q = arc_pts(x, y, 10.0, PI, PI * 1.5, 6);
        q.push(vec2(x, y));
        g.fill(&q, "#4a9be0");
        g.stroke(&q, 1.5, INK, true);
        g.ln(x, y - 10.0, x, y - 15.0, 1.8, INK);
        let s = (t * 40.0).cos() * 13.0;
        g.fe(x, y - 16.0, s.abs() + 1.0, 2.6, "#f3d23c", 1.5);
    }
    fn jet(&self, g: &mut Painter, x: f32, y: f32, fire: bool) {
        for o in [0.0, 9.0] {
            g.fr(x + o, y, 9.0, 22.0, 4.0, "#b9c4cc", 1.8);
        }
        g.stroke(&[vec2(x, y + 3.0), vec2(x + 9.0, y - 6.0), vec2(x + 18.0, y + 3.0)], 1.8, INK, false);
        if fire {
            for o in [4.5, 13.5] {
                let f = 10.0 + chance() * 10.0;
                let tri = [vec2(x + o - 4.0, y + 22.0), vec2(x + o, y + 22.0 + f), vec2(x + o + 4.0, y + 22.0)];
                g.fill(&tri, if chance() < 0.5 { "#f39b2d" } else { "#e8504a" });
                g.stroke(&tri, 1.4, INK, true);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn bubu(&self, g: &mut Painter, x: f32, y: f32, face: f32, sq: f32, shooting: bool, flip: f32, pu: Item) {
        g.save();
        g.tr(x, y);
        if flip > 0.0 {
            g.tr(0.0, -28.0);
            g.rot((1.0 - flip) * TAU * face);
            g.tr(0.0, 28.0);
        }
        g.sc(1.0 + sq * 0.12, 1.0 - sq * 0.12);
        let dd = self.sv.dudu;
        let b = if dd { "#d49b86" } else { "#ffffff" };
        if pu == Item::Jet {
            g.save();
            g.tr(-face * 16.0 - 9.0, -38.0);
            self.jet(g, 0.0, 0.0, true);
            g.restore();
        }
        let ex = face * 1.5;
        for s in [-1.0f32, 1.0] {
            g.fe(s * 7.0, -2.0, 6.0, 3.5, b, 2.0);
        }
        g.fr(-13.0, -20.0, 26.0, 18.0, 8.0, b, 2.2);
        for s in [-1.0f32, 1.0] {
            g.fcirc(s * 16.0, -50.0, 7.5, if dd { b } else { "#4a2822" }, 2.0);
            if dd {
                g.fcirc(s * 16.0, -50.0, 3.6, "#4a2822", 0.0);
            }
        }
        g.fe(0.0, -33.0, 23.0, 19.5, b, 2.4);
        let (cheek, cr) = if dd { ("#f6c27a", 5.6) } else { ("#f7b2ae", 4.8) };
        g.fcirc(-15.0 + ex * 0.5, -26.0, cr, cheek, 0.0);
        g.fcirc(15.0 + ex * 0.5, -26.0, cr, cheek, 0.0);
        if !dd {
            g.fill(&[vec2(0.0, -14.0), vec2(-7.0, -18.0), vec2(-7.0, -10.0)], "#4a2822");
            g.fill(&[vec2(0.0, -14.0), vec2(7.0, -18.0), vec2(7.0, -10.0)], "#4a2822");
            g.fcirc(0.0, -14.0, 2.4, "#4a2822", 0.0);
        }
        let up = if shooting { -4.0 } else { 0.0 };
        for s in [-1.0f32, 1.0] {
            g.fe(s * 13.0, -17.0 + up, 4.5, 5.5, b, 2.0);
        }
        if self.time % 3.4 < 0.12 {
            g.ln(-11.0 + ex, -33.0, -5.0 + ex, -33.0, 2.0, INK);
            g.ln(5.0 + ex, -33.0, 11.0 + ex, -33.0, 2.0, INK);
        } else {
            g.fcirc(-8.0 + ex, -33.0, 2.8, INK, 0.0);
            g.fcirc(8.0 + ex, -33.0, 2.8, INK, 0.0);
            g.fcirc(-7.2 + ex, -34.0, 0.9, "#ffffff", 0.0);
            g.fcirc(8.8 + ex, -34.0, 0.9, "#ffffff", 0.0);
        }
        if shooting {
            g.fe(ex, -27.0, 2.6, 3.0, INK, 0.0);
        } else {
            let mut s = arc_pts(-1.9 + ex, -29.0, 1.9, 0.0, PI, 6);
            s.extend(arc_pts(1.9 + ex, -29.0, 1.9, 0.0, PI, 6));
            g.stroke(&s, 1.6, INK, false);
        }
        if pu == Item::Prop {
            self.hat(g, 0.0, -51.0, self.time);
        }
        g.restore();
    }

    fn monster(&self, g: &mut Painter, m: &Mon) {
        g.save();
        g.tr(m.x + m.bx, m.y);
        if m.dead {
            g.rot(m.vy * 0.002);
        }
        if !m.bat {
            g.fe(0.0, 8.0, 28.0, 14.0, "#6fae3a", 2.0);
            g.fe(0.0, -4.0, 26.0, 16.0, "#9bd25a", 2.2);
            let mut i = -22.0f32;
            while i <= 22.0 {
                g.ln(i, -17.0 + i.abs() * 0.2, i + 1.0, -23.0 + i.abs() * 0.2, 1.2, INK);
                i += 4.0;
            }
            for e in [(-11.0, 6.0), (11.0, 6.0), (0.0, 4.0)] {
                let yy = -5.0 - if e.1 < 5.0 { 6.0 } else { 0.0 };
                g.fcirc(e.0, yy, e.1, "#f3d23c", 1.6);
                g.fcirc(e.0, yy, 1.8, INK, 0.0);
            }
            for s in [-1.0f32, 1.0] {
                g.fr(s * 8.0 - 2.0, 4.0, 5.0, 6.0, 1.0, "#ffffff", 1.2);
            }
        } else {
            let f = (m.ph * 4.0).sin() * 0.4;
            for s in [-1.0f32, 1.0] {
                g.save();
                g.sc(s, 1.0);
                g.rot(f);
                let mut wp = vec![vec2(8.0, -4.0)];
                wp.extend(quad_pts(vec2(8.0, -4.0), vec2(28.0, -16.0), vec2(32.0, -6.0), 6));
                wp.push(vec2(30.0, 6.0));
                wp.extend(quad_pts(vec2(30.0, 6.0), vec2(20.0, 4.0), vec2(8.0, 6.0), 6));
                wp.dedup();
                g.fill(&wp, "#5b8fd0");
                g.stroke(&wp, 2.0, INK, true);
                for i in [14.0f32, 19.0, 24.0] {
                    g.ln(i, -8.0 + (i - 14.0) * 0.1, i, 5.0, 1.0, INK);
                }
                g.restore();
            }
            g.fe(0.0, 0.0, 12.0, 13.0, "#6cb6f0", 2.2);
            for s in [-1.0f32, 1.0] {
                g.fcirc(s * 4.5, -3.0, 3.8, "#f3d23c", 1.3);
                g.fcirc(s * 4.5, -3.0, 1.4, INK, 0.0);
            }
            g.ln(-5.0, 6.0, 5.0, 6.0, 1.6, INK);
            g.ln(-4.0, 13.0, -5.0, 19.0, 1.8, INK);
            g.ln(4.0, 13.0, 5.0, 19.0, 1.8, INK);
        }
        g.restore();
    }

    fn coin(&self, g: &mut Painter, c: &Coin) {
        let w = (self.time * 4.0 + c.ph).cos().abs();
        g.save();
        g.tr(c.x, c.y + (self.time * 3.0 + c.ph).sin() * 2.0);
        g.fe(0.0, 0.0, 8.0 * w + 1.5, 8.0, "#f3d23c", 1.8);
        if w > 0.4 {
            g.ln(-1.0, -3.5, -1.0, 3.5, 1.5, INK);
        }
        g.restore();
    }

    fn paint_parts(&self, g: &mut Painter) {
        for p in &self.parts {
            let k = (p.life / p.max).clamp(0.0, 1.0);
            g.save();
            g.ga *= k.min(1.0);
            let mut c = p.col;
            c.a *= g.ga;
            if p.star {
                let (s, co) = p.rot.sin_cos();
                let pts: Vec<Vec2> = star_pts(p.size).iter().map(|v| vec2(p.x + v.x * co - v.y * s, p.y + v.x * s + v.y * co)).collect();
                g.fill_c(&pts, c);
            } else {
                g.fill_c(&ell_pts(p.x, p.y, p.size * (0.4 + k * 0.6), p.size * (0.4 + k * 0.6)), c);
            }
            g.restore();
        }
        for r in &self.rings {
            let mut c = r.col;
            c.a *= (1.0 - r.t) * g.ga;
            g.stroke_c(&ell_pts(r.x, r.y, 10.0 + r.t * 50.0, 10.0 + r.t * 50.0), 3.0 * (1.0 - r.t) + 0.5, c, true);
        }
        for f in &self.ftxt {
            g.save();
            g.ga *= 1.0 - f.t;
            g.txt(&f.s, f.x, f.y, 22.0, INK, 1, 4.0);
            g.restore();
        }
    }

    fn paint_world(&self, g: &mut Painter) {
        g.save();
        g.tr((chance() - 0.5) * self.shake, (chance() - 0.5) * self.shake);
        self.paper(g);
        for p in &self.plats {
            self.plat(g, p);
        }
        for c in &self.coins {
            self.coin(g, c);
        }
        for m in &self.mons {
            self.monster(g, m);
        }
        let bnk = self.night_k();
        for b in &self.bullets {
            g.save();
            g.ga *= 0.3;
            g.fe(b.x, b.y + 8.0, 2.5, 7.0, if bnk < 0.5 { INK } else { "#fff8e0" }, 0.0);
            g.restore();
            if bnk > 0.3 {
                g.fcirc(b.x, b.y, 5.0, "#fff8e0", 0.0);
            }
            g.fcirc(b.x, b.y, 3.5, INK, 0.0);
        }
        self.paint_parts(g);
        self.bubu(g, self.p.x, self.p.y, self.p.face, self.p.sq, self.p.shoot_t > 0.0, self.p.flip, self.p.pu);
        g.restore();
        if self.flash > 0.0 {
            g.rect(0.0, 0.0, self.w, self.h, Color::new(1.0, 1.0, 1.0, self.flash * 0.6));
        }
    }

    fn hud(&mut self, g: &mut Painter) {
        let w = self.w;
        g.rect(0.0, 0.0, w, 40.0, hx("#d9e2e6cc"));
        let mut wave = vec![];
        let mut xx = 0.0;
        while xx <= w {
            wave.push(vec2(xx, 40.0 + (xx * 0.7).sin() * 0.8));
            xx += 12.0;
        }
        g.stroke(&wave, 2.4, INK, false);
        g.txt(&(self.score.floor() as i64).to_string(), 12.0, 22.0, 28.0, INK, 0, 0.0);
        g.rect(w - 30.0, 11.0, 4.0, 18.0, hx(INK));
        g.rect(w - 21.0, 11.0, 4.0, 18.0, hx(INK));
        self.btns.push(Btn { x: w - 44.0, y: 0.0, w: 44.0, h: 40.0, id: 10 });
        g.fcirc(238.0, 21.0, 7.0, "#f3d23c", 1.8);
        g.txt(&self.run_coins.to_string(), 250.0, 22.0, 22.0, INK, 0, 0.0);
        g.txt(&format!("{} m", self.alt_m()), 12.0, 56.0, 18.0, "#7a6355", 0, 4.0);
        if self.combo >= 3 {
            g.save();
            g.tr(14.0, 82.0);
            let cs = 1.0 + self.combo_pop * 0.35;
            g.sc(cs, cs);
            g.rot(-0.05);
            g.txt(&format!("combo x{}", self.combo), 0.0, 0.0, 20.0, "#e8504a", 0, 5.0);
            g.restore();
            g.txt(&format!("puntos ×{:.2}", self.combo_mult()), 14.0, 102.0, 15.0, "#7a6355", 0, 4.0);
        }
        if self.timed() {
            let pulse = 1.0 + if self.tl < 10.0 { (self.time * 8.0).sin().abs() * 0.15 } else { 0.0 };
            g.save();
            g.tr(w / 2.0, 66.0);
            g.sc(pulse, pulse);
            g.txt(&format!("{}s", self.tl.ceil() as i32), 0.0, 0.0, 32.0, if self.tl < 10.0 { "#e8504a" } else { INK }, 1, 6.0);
            g.restore();
        }
        if self.p.pu != Item::None {
            let mx = if self.p.pu == Item::Jet { 2.6 } else { 3.2 };
            let k = (self.p.put / mx).max(0.0);
            g.fr(110.0, 16.0, 110.0, 10.0, 5.0, "#ffffff", 0.0);
            g.fr(110.0, 16.0, 110.0 * k, 10.0, 5.0, if self.p.pu == Item::Jet { "#f39b2d" } else { "#4a9be0" }, 0.0);
            g.stroke(&rr_pts(110.0, 16.0, 110.0, 10.0, 5.0), 1.8, INK, true);
        }
    }

    fn pad(&self, g: &mut Painter) {
        if !self.touch_ui {
            return;
        }
        let y = self.h - 32.0;
        g.save();
        g.ga = 0.12;
        let (w, h) = (self.w, self.h);
        for x in [w / 3.0, w * 2.0 / 3.0] {
            let mut yy = 48.0;
            while yy < h {
                g.ln(x, yy, x, yy + 6.0, 1.6, INK);
                yy += 12.0;
            }
        }
        for (z, x) in [(-1i32, w / 6.0), (0, w / 2.0), (1, w * 5.0 / 6.0)] {
            let on = self.zones[(z + 1) as usize];
            g.ga = if on { 0.75 } else { 0.28 };
            g.fcirc(x, y, 19.0, if on { "#f3d23c" } else { "#ffffff" }, 2.0);
            if z != 0 {
                let zf = z as f32;
                g.stroke(&[vec2(x - zf * 4.0, y - 8.0), vec2(x + zf * 4.0, y), vec2(x - zf * 4.0, y + 8.0)], 3.0, INK, false);
            } else {
                g.fcirc(x, y, 4.0, INK, 0.0);
            }
        }
        g.restore();
    }

    fn button(&mut self, g: &mut Painter, label: &str, x: f32, y: f32, w: f32, id: u8, col: &str) {
        let h = 42.0;
        let y = y + (self.time * 3.0 + x * 0.05).sin() * 1.5;
        g.fr(x, y + 4.0, w, h, 14.0, "#3b242033", 0.0);
        g.fr(x, y, w, h, 14.0, col, 0.0);
        g.fr(x + 4.0, y + h - 9.0, w - 8.0, 5.0, 3.0, "#00000011", 0.0);
        g.stroke(&rr_pts(x, y, w, h, 14.0), 2.4, INK, true);
        g.txt(label, x + w / 2.0, y + h / 2.0 + 1.0, 26.0, INK, 1, 0.0);
        self.btns.push(Btn { x, y, w, h, id });
    }
    fn title(&self, g: &mut Painter, s: &str, x: f32, y: f32, size: f32, rot: f32) {
        g.save();
        g.tr(x, y);
        g.rot(rot);
        g.txt(s, 0.0, 0.0, size, INK, 1, 7.0);
        g.restore();
    }
    fn sel_row(&mut self, g: &mut Painter, ox: f32, y: f32, label: &str, id_l: u8, id_r: u8) {
        let cx = ox + BASE_W / 2.0;
        g.txt(label, cx, y, 22.0, INK, 1, 0.0);
        for (s, id) in [(-1.0f32, id_l), (1.0, id_r)] {
            let x = cx + s * 120.0;
            g.fcirc(x, y, 14.0, "#ffffff", 2.0);
            g.stroke(&[vec2(x - s * 3.0, y - 6.0), vec2(x + s * 3.0, y), vec2(x - s * 3.0, y + 6.0)], 2.4, INK, false);
            self.btns.push(Btn { x: x - 20.0, y: y - 18.0, w: 40.0, h: 36.0, id });
        }
    }

    fn menu(&mut self, g: &mut Painter) {
        let ox = (self.w - BASE_W) / 2.0;
        let cx = self.w / 2.0;
        let wob = (self.time * 2.0).sin() * 0.02;
        let name = if self.sv.dudu { "dudu" } else { "bubu" };
        self.title(g, name, cx, 120.0, 74.0, -0.06 + wob);
        self.title(g, "jump", cx, 178.0, 70.0, 0.04 - wob);
        g.fcirc(ox + 268.0, 98.0, 6.0, "#f7b2ae", 0.0);
        if chance() < 0.08 {
            let (x, y) = (rnd(ox + 40.0, ox + BASE_W - 40.0), rnd(80.0, 220.0));
            self.fx(x, y, 1, &STARC, 20.0, -20.0, 1.2, 0.8, true, 0.0);
        }
        self.paint_parts(g);
        let by = 430.0 - (self.time * 3.2).sin().abs() * 120.0;
        let p0 = self.mk(cx - 29.0, 432.0, PT::G);
        self.plat(g, &p0);
        self.bubu(g, cx, by, 1.0, if by > 420.0 { 1.0 } else { 0.0 }, false, self.swap_t, Item::None);
        let bat = Mon { x: ox + 80.0, y: 300.0 + (self.time * 2.0).sin() * 10.0, bx: 0.0, bat: true, w: 56.0, h: 34.0, ph: self.time * 2.0, dead: false, vy: 0.0 };
        self.monster(g, &bat);
        let pb = self.mk(ox + 276.0, 370.0, PT::B);
        self.plat(g, &pb);
        self.btns.push(Btn { x: cx - 32.0, y: by - 66.0, w: 64.0, h: 74.0, id: 5 });
        g.save();
        g.tr(ox + 272.0, 340.0);
        g.rot(0.08);
        g.txt("toca al osito", 8.0, 0.0, 16.0, "#7a6355", 1, 0.0);
        g.stroke(&[vec2(-62.0, 0.0), vec2(-48.0, 0.0)], 1.6, "#7a6355", false);
        g.stroke(&[vec2(-56.0, -4.0), vec2(-62.0, 0.0), vec2(-56.0, 4.0)], 1.6, "#7a6355", false);
        g.restore();
        let mode = MODES[self.sv.mode];
        self.sel_row(g, ox, 462.0, &format!("modo: {}", mode.1), 1, 2);
        self.sel_row(g, ox, 500.0, &format!("nivel: {}", DIFS[self.sv.dif].0), 3, 4);
        self.button(g, "jugar", cx - 100.0, 540.0, 200.0, 0, "#86d443");
        g.txt(&format!("mejor: {}   monedas: {}", self.sv.best(), self.sv.coins), cx, 226.0, 19.0, INK, 1, 0.0);
        g.txt(mode.2, cx, 248.0, 15.0, "#7a6355", 1, 0.0);
        if !self.touch_ui {
            g.txt("flechas o A D mover · espacio disparar", cx, 268.0, 14.0, "#7a6355", 1, 0.0);
        }
    }

    fn over_screen(&mut self, g: &mut Painter) {
        let cx = self.w / 2.0;
        let k = (self.over_t * 3.0).min(1.0);
        g.save();
        g.ga = k;
        g.rect(0.0, 0.0, self.w, self.h, Color::new(0.97, 0.94, 0.89, 0.55));
        let pop = 1.0 + (1.0 - self.over_t * 4.0).max(0.0) * 0.4;
        g.save();
        g.tr(cx, 130.0);
        g.sc(pop, pop);
        let t = if self.timed() && self.tl <= 0.0 { "¡tiempo!" } else { "¡se acabó!" };
        self.title(g, t, 0.0, 0.0, 52.0, -0.05);
        g.restore();
        g.txt(&format!("puntos: {}   ·   {} m", self.score.floor() as i64, self.alt_m()), cx, 180.0, 24.0, INK, 1, 0.0);
        g.txt(&format!("mejor: {}", self.sv.best()), cx, 206.0, 22.0, INK, 1, 0.0);
        g.txt(&format!("monedas: +{}   ·   combo máx: x{}", self.run_coins, self.run_max_combo), cx, 230.0, 20.0, INK, 1, 0.0);
        if self.new_best {
            g.save();
            g.tr(cx, 256.0);
            g.rot((self.time * 5.0).sin() * 0.06);
            g.txt("¡nuevo récord!", 0.0, 0.0, 22.0, "#e8504a", 1, 0.0);
            g.restore();
        }
        self.bubu(g, cx, 330.0 + (self.time * 4.0).sin() * 3.0, 1.0, 0.0, false, 0.0, Item::None);
        self.button(g, "otra vez", cx - 85.0, 400.0, 170.0, 0, "#86d443");
        self.button(g, "menú", cx - 85.0, 452.0, 170.0, 6, "#ffffff");
        g.restore();
    }

    fn pause_screen(&mut self, g: &mut Painter) {
        let cx = self.w / 2.0;
        g.rect(0.0, 0.0, self.w, self.h, Color::new(0.97, 0.94, 0.89, 0.7));
        self.title(g, "pausa", cx, 200.0, 60.0, -0.04);
        self.button(g, "seguir", cx - 85.0, 300.0, 170.0, 11, "#86d443");
        self.button(g, "menú", cx - 85.0, 352.0, 170.0, 6, "#ffffff");
    }

    fn act(&mut self, id: u8) {
        match id {
            0 => self.start(),
            1 => self.sv.mode = (self.sv.mode + MODES.len() - 1) % MODES.len(),
            2 => self.sv.mode = (self.sv.mode + 1) % MODES.len(),
            3 => self.sv.dif = (self.sv.dif + 2) % 3,
            4 => self.sv.dif = (self.sv.dif + 1) % 3,
            5 => {
                self.sv.dudu = !self.sv.dudu;
                self.swap_t = 1.0;
            }
            6 => self.state = State::Menu,
            10 => self.state = State::Pause,
            11 => self.state = State::Play,
            _ => {}
        }
        if (1..=5).contains(&id) {
            self.sv.write();
        }
    }

    fn click(&mut self, p: Vec2) -> bool {
        for i in (0..self.btns.len()).rev() {
            let b = &self.btns[i];
            if p.x >= b.x && p.x <= b.x + b.w && p.y >= b.y && p.y <= b.y + b.h {
                let id = b.id;
                self.act(id);
                return true;
            }
        }
        false
    }
}

fn window_conf() -> Conf {
    let fs = std::env::args().any(|a| a == "--fullscreen" || a == "-f");
    Conf {
        window_title: "Bubu Jump".to_owned(),
        window_width: 540,
        window_height: 960,
        fullscreen: fs || cfg!(target_os = "android"),
        high_dpi: true,
        sample_count: 2,
        icon: None,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let font = load_ttf_font_from_bytes(include_bytes!("GochiHand-Regular.ttf")).expect("fuente");
    let mut g = Game::new();
    let mut pt = Painter { m: Affine2::IDENTITY, stack: vec![], ga: 1.0, font, px: 1.0 };
    let mut acc = 0.0f32;
    let shot_path = std::env::var("BUBU_SHOT").ok();
    let mut frames = 0u32;

    loop {
        let (sw, sh) = (screen_width(), screen_height());
        let scale = (sw / BASE_W).min(sh / BASE_H);
        let (nw, nh) = (sw / scale, sh / scale);
        if (nw - g.w).abs() > 0.5 || (nh - g.h).abs() > 0.5 {
            g.w = nw;
            g.h = nh;
            if g.state == State::Play {
                g.setup_chains();
            }
        }
        pt.px = 1.0;
        set_default_camera();
        let to_world = |p: Vec2| p / scale;

        let dt = get_frame_time().min(1.0 / 20.0);
        g.time += dt;
        g.swap_t = (g.swap_t - dt * 2.0).max(0.0);

        // ---- entrada táctil ----
        let ts = touches();
        g.touch_dir = 0.0;
        g.touch_fire = false;
        g.zones = [false; 3];
        if !ts.is_empty() {
            g.touch_ui = true;
        }
        for t in &ts {
            let wp = to_world(t.position);
            if t.phase == TouchPhase::Started && g.state != State::Play && g.click(wp) {
                continue;
            }
            if g.state == State::Play {
                if t.phase == TouchPhase::Started && wp.y < 44.0 && g.click(wp) {
                    continue;
                }
                if matches!(t.phase, TouchPhase::Started | TouchPhase::Moved | TouchPhase::Stationary) {
                    let z = ((wp.x / g.w * 3.0) as i32).clamp(0, 2);
                    g.zones[z as usize] = true;
                    match z {
                        0 => g.touch_dir -= 1.0,
                        2 => g.touch_dir += 1.0,
                        _ => g.touch_fire = true,
                    }
                }
            }
        }
        if ts.is_empty() && is_mouse_button_pressed(MouseButton::Left) {
            let wp = to_world(mouse_position().into());
            if !g.click(wp) && g.state == State::Play {
                g.shoot();
            }
        }
        if !g.touch_fire {
            g.fire_t = 0.0;
        }

        // ---- teclado ----
        match g.state {
            State::Menu => {
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    g.start();
                }
                if is_key_pressed(KeyCode::Left) {
                    g.act(1);
                }
                if is_key_pressed(KeyCode::Right) {
                    g.act(2);
                }
                if is_key_pressed(KeyCode::Up) {
                    g.act(4);
                }
                if is_key_pressed(KeyCode::Down) {
                    g.act(3);
                }
                if is_key_pressed(KeyCode::Tab) {
                    g.act(5);
                }
                if is_key_pressed(KeyCode::Escape) {
                    break;
                }
            }
            State::Play => {
                if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    g.shoot();
                }
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P) {
                    g.state = State::Pause;
                }
            }
            State::Pause => {
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Enter) {
                    g.state = State::Play;
                }
            }
            State::Over => {
                if g.over_t > 0.4 && (is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space)) {
                    g.start();
                }
                if is_key_pressed(KeyCode::Escape) {
                    g.state = State::Menu;
                }
            }
        }

        // ---- simulación a paso fijo ----
        if g.state == State::Play {
            acc += dt;
            let h = 1.0 / 120.0;
            while acc >= h && g.state == State::Play {
                g.step(h);
                acc -= h;
            }
        } else {
            acc = 0.0;
        }
        if g.state == State::Over {
            g.over_t += dt;
        }
        if g.state != State::Pause {
            g.update_fx(dt);
            g.shake = (g.shake - dt * 30.0).max(0.0);
            g.flash = (g.flash - dt * 3.0).max(0.0);
        }

        // ---- dibujo ----
        clear_background(hx("#2b2420"));
        g.btns.clear();
        pt.m = Affine2::from_scale(vec2(scale, scale));
        pt.ga = 1.0;
        pt.stack.clear();
        match g.state {
            State::Menu => {
                g.paper(&mut pt);
                g.menu(&mut pt);
            }
            _ => {
                g.paint_world(&mut pt);
                if g.state == State::Play {
                    g.hud(&mut pt);
                    g.pad(&mut pt);
                }
                if g.state == State::Pause {
                    g.pause_screen(&mut pt);
                }
                if g.state == State::Over {
                    g.over_screen(&mut pt);
                }
            }
        }

        // captura para pruebas automáticas: BUBU_SHOT=archivo.png [BUBU_STATE=play|over]
        if let Some(path) = &shot_path {
            frames += 1;
            if frames == 2 {
                match std::env::var("BUBU_STATE").as_deref() {
                    Ok("play") | Ok("over") => {
                        g.start();
                        g.score = 6000.0;
                        for _ in 0..240 {
                            g.p.vy = g.p.vy.min(-300.0);
                            g.step(1.0 / 60.0);
                        }
                    }
                    _ => {}
                }
                if std::env::var("BUBU_STATE").as_deref() == Ok("over") {
                    g.game_over();
                    g.over_t = 2.0;
                }
            }
            if frames == 6 {
                get_screen_data().export_png(path);
                break;
            }
        }
        next_frame().await;
    }
}
