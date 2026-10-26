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
const MODES: [(&str, &str, &str); 4] = [
    ("classic", "clásico", "sube lo más alto que puedas"),
    ("time", "contrarreloj", "60 s para subir lo más alto posible"),
    ("zen", "zen", "sin enemigos ni eventos peligrosos"),
    ("chaos", "caos", "eventos sin parar y más fuertes"),
];
// (nombre, gap, monstruos, monedas, espera de eventos, intensidad)
const DIFS: [(&str, f32, f32, f32, f32, f32); 3] = [
    ("fácil", 0.85, 0.5, 1.3, 1.4, 0.75),
    ("normal", 1.0, 1.0, 1.0, 1.0, 1.0),
    ("difícil", 1.1, 1.8, 0.7, 0.7, 1.2),
];
const MAG_R: f32 = 170.0;
const MAG_T: f32 = 10.0;
// tienda: (id, nombre, precio, color)
const SHOP_HAT: [(&str, &str, i64, &str); 6] = [("party", "gorro fiesta", 30, ""), ("crown", "corona", 120, ""), ("top", "sombrero copa", 60, ""), ("halo", "aureola", 90, ""), ("cap", "gorra", 40, ""), ("flower", "flor", 25, "")];
const SHOP_ACC: [(&str, &str, i64, &str); 4] = [("shades", "gafas de sol", 50, ""), ("round", "gafas", 35, ""), ("stache", "bigote", 45, ""), ("scarf", "bufanda", 55, "")];
const SHOP_COL: [(&str, &str, i64, &str); 4] = [("pink", "rosa", 40, "#f7b2ae"), ("blue", "azul", 40, "#9fd6f2"), ("mint", "menta", 40, "#a8e6b8"), ("gold", "dorado", 150, "#f3d23c")];
const SHOP_TRL: [(&str, &str, i64, &str); 3] = [("star", "estrellas", 70, ""), ("conf", "confeti", 60, ""), ("bub", "burbujas", 50, "")];
const SHOP_POW: [(&str, &str, i64, &str); 3] = [("magnet", "imán", 60, ""), ("shield", "escudo", 50, ""), ("respawn", "reaparecer", 70, "")];
const TABS: [(&str, &str); 5] = [("hat", "gorros"), ("acc", "extras"), ("col", "colores"), ("trl", "estelas"), ("pow", "poderes")];
fn shop_items(cat: &str) -> &'static [(&'static str, &'static str, i64, &'static str)] {
    match cat {
        "hat" => &SHOP_HAT,
        "acc" => &SHOP_ACC,
        "col" => &SHOP_COL,
        "trl" => &SHOP_TRL,
        _ => &SHOP_POW,
    }
}
fn pow_desc(id: &str) -> &'static str {
    match id {
        "magnet" => "10 s atrae monedas · tecla X",
        "shield" => "te protege de 1 golpe",
        _ => "devuelve plataformas destruidas",
    }
}
// clima
#[derive(Clone, Copy, PartialEq, Debug)]
enum Wx { Rain, Storm, Meteor, Acid, Snow, Wind, Hail, Birds, CoinRain, Fog, Moon, Quake }
const WX_ALL: [Wx; 12] = [Wx::Rain, Wx::Storm, Wx::Meteor, Wx::Acid, Wx::Snow, Wx::Wind, Wx::Hail, Wx::Birds, Wx::CoinRain, Wx::Fog, Wx::Moon, Wx::Quake];
const WX_ZEN: [Wx; 5] = [Wx::Snow, Wx::Wind, Wx::Fog, Wx::Moon, Wx::CoinRain];
fn wx_name(w: Wx) -> &'static str {
    match w {
        Wx::Rain => "lluvia", Wx::Storm => "tormenta eléctrica", Wx::Meteor => "lluvia de meteoros", Wx::Acid => "lluvia ácida",
        Wx::Snow => "nieve", Wx::Wind => "ventarrón", Wx::Hail => "granizo", Wx::Birds => "bandada", Wx::CoinRain => "lluvia de monedas",
        Wx::Fog => "niebla", Wx::Moon => "gravedad ligera", Wx::Quake => "temblor",
    }
}
fn wx_tint(w: Wx) -> (f32, f32, f32) {
    match w {
        Wx::Rain => (60.0, 90.0, 150.0), Wx::Storm => (40.0, 40.0, 80.0), Wx::Acid => (90.0, 140.0, 40.0), Wx::Snow => (200.0, 220.0, 240.0),
        Wx::Wind => (190.0, 160.0, 110.0), Wx::Meteor => (200.0, 90.0, 40.0), Wx::Hail => (150.0, 190.0, 220.0), Wx::Birds => (120.0, 110.0, 100.0),
        Wx::CoinRain => (240.0, 200.0, 60.0), Wx::Fog => (200.0, 200.0, 205.0), Wx::Moon => (110.0, 90.0, 190.0), Wx::Quake => (120.0, 90.0, 60.0),
    }
}
// misiones: (clave, texto, metas, premio base)
const MISSIONS: [(&str, &str, &[i64], i64); 7] = [
    ("jumps", "salta en {} plataformas", &[50, 80, 120], 15),
    ("coins", "junta {} monedas", &[20, 35, 50], 20),
    ("kills", "derrota {} enemigos", &[3, 5, 8], 25),
    ("pu", "usa {} potenciadores", &[1, 2, 3], 20),
    ("runs", "juega {} partidas", &[3, 5], 15),
    ("alt", "llega a {} m en una partida", &[300, 500, 800], 30),
    ("combo", "haz un combo x{}", &[10, 15, 20], 25),
];
// logros: (id, nombre, descripción, estadística, meta, premio)
const ACHS: [(&str, &str, &str, &str, i64, i64); 14] = [
    ("m500", "500 m", "llega a 500 m", "alt_best", 500, 20),
    ("m1000", "1000 m", "llega a 1000 m", "alt_best", 1000, 50),
    ("m2500", "2500 m", "llega a 2500 m", "alt_best", 2500, 120),
    ("c100", "ahorrador", "junta 100 monedas", "coins", 100, 20),
    ("c1000", "millonario", "junta 1000 monedas", "coins", 1000, 100),
    ("j1000", "saltarín", "salta 1000 veces", "jumps", 1000, 40),
    ("k25", "cazador", "vence 25 enemigos", "kills", 25, 40),
    ("x15", "en racha", "haz un combo x15", "combo_best", 15, 30),
    ("x40", "imparable", "haz un combo x40", "combo_best", 40, 80),
    ("t300", "a contrarreloj", "300 m en 60 s", "time_alt_best", 300, 50),
    ("d3", "constante", "racha de 3 días", "streak_best", 3, 30),
    ("d7", "fiel", "racha de 7 días", "streak_best", 7, 80),
    ("ch10", "cazatesoros", "abre 10 cofres", "chests", 10, 60),
    ("mo4", "explorador", "juega los 4 modos", "modes", 4, 30),
];
const STREAK_R: [i64; 7] = [5, 10, 15, 20, 30, 40, 60];
#[derive(Clone)]
struct Mis { k: String, n: i64, p: i64, r: i64, c: bool }
fn mis_text(m: &Mis) -> String {
    MISSIONS.iter().find(|d| d.0 == m.k).map(|d| d.1.replace("{}", &m.n.to_string())).unwrap_or_default()
}
fn local_offset() -> i64 {
    #[cfg(unix)]
    unsafe {
        let t = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&t, &mut tm);
        return tm.tm_gmtoff as i64;
    }
    #[allow(unreachable_code)]
    0
}
fn now_secs() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0) + local_offset()
}
fn today() -> i64 {
    now_secs().div_euclid(86400)
}
fn secs_to_midnight() -> i64 {
    86400 - now_secs().rem_euclid(86400)
}
const K_DOT: u8 = 0;
const K_STAR: u8 = 1;
const K_LEAF: u8 = 2;
const K_GUST: u8 = 3;
const K_CONF: u8 = 4;

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


// ---------- sonido: port de synth.gd (todo generado por código) ----------
mod synth {
    use macroquad::audio::{load_sound_from_bytes, play_sound, set_sound_volume, PlaySoundParams, Sound};
    use std::collections::HashMap;
    use std::f32::consts::TAU;
    pub const SR: usize = 22050;
    const MEL: [&str; 32] = ["E5", "", "G5", "", "C6", "B5", "G5", "", "A5", "", "E5", "", "C5", "", "E5", "G5",
        "F5", "", "A5", "", "C6", "", "A5", "F5", "G5", "", "B5", "", "D6", "B5", "G5", ""];
    const BAS: [&str; 32] = ["C2", "", "C3", "", "C2", "C2", "C3", "", "A1", "", "A2", "", "A1", "A1", "A2", "",
        "F1", "", "F2", "", "F1", "F1", "F2", "", "G1", "", "G2", "", "G1", "G1", "B1", "D2"];
    const DRM: &[u8] = b"khshkksh";
    const MUSVOL: f32 = 0.35;
    const SFXVOL: f32 = 0.8;
    #[derive(Clone, Copy)]
    enum Wv { Sine, Square, Saw, Tri }
    #[derive(Clone, Copy)]
    enum Ft { Low, High, Band }

    struct Gen { buf: Vec<f32>, seed: u32 }
    impl Gen {
        fn new(dur: f32, seed: u32) -> Gen {
            Gen { buf: vec![0.0; (dur * SR as f32) as usize], seed }
        }
        fn rnd(&mut self) -> f32 {
            self.seed ^= self.seed << 13;
            self.seed ^= self.seed >> 17;
            self.seed ^= self.seed << 5;
            (self.seed as f32 / u32::MAX as f32) * 2.0 - 1.0
        }
        fn tone(&mut self, t0: f32, ty: Wv, f0: f32, f1: f32, dur: f32, vol: f32) {
            let n0 = (t0 * SR as f32) as usize;
            let n = (dur * SR as f32) as usize;
            let ratio = f1 / f0;
            let eb = 0.001 / vol;
            let mut ph = 0.0f32;
            for i in 0..n {
                if n0 + i >= self.buf.len() {
                    break;
                }
                let u = i as f32 / n as f32;
                ph += f0 * ratio.powf(u) / SR as f32;
                let p = ph - ph.floor();
                let s = match ty {
                    Wv::Sine => (TAU * p).sin(),
                    Wv::Square => if p < 0.5 { 1.0 } else { -1.0 },
                    Wv::Saw => 2.0 * p - 1.0,
                    Wv::Tri => 4.0 * (p - 0.5).abs() - 1.0,
                };
                self.buf[n0 + i] += s * vol * eb.powf(u);
            }
        }
        #[allow(clippy::too_many_arguments)]
        fn noise(&mut self, t0: f32, dur: f32, vol: f32, ft: Ft, ff: f32, brown: bool, q: f32) {
            let n = (dur * SR as f32) as usize;
            let mut l = 0.0;
            let src: Vec<f32> = (0..n).map(|_| {
                let w = self.rnd();
                if brown {
                    l = (l + 0.02 * w) / 1.02;
                    l * 3.5
                } else {
                    w
                }
            }).collect();
            let out = biquad(&src, ft, ff, q);
            let n0 = (t0 * SR as f32) as usize;
            let eb = 0.001 / vol;
            for i in 0..n {
                if n0 + i >= self.buf.len() {
                    break;
                }
                self.buf[n0 + i] += out[i] * vol * eb.powf(i as f32 / n as f32);
            }
        }
        fn arp(&mut self, t: f32, ns: &[&str], st: f32, ty: Wv, vol: f32, dur: f32) {
            for (i, n) in ns.iter().enumerate() {
                let f = nf(n);
                self.tone(t + i as f32 * st, ty, f, f, dur, vol);
            }
        }
    }
    fn nf(n: &str) -> f32 {
        let b = n.as_bytes();
        let semis = match b[0] { b'C' => -9, b'D' => -7, b'E' => -5, b'F' => -4, b'G' => -2, b'A' => 0, _ => 2 };
        let oct = (b[1] - b'0') as i32;
        440.0 * 2f32.powf((semis + (oct - 4) * 12) as f32 / 12.0)
    }
    fn coef(ft: Ft, fc: f32, q: f32) -> [f32; 5] {
        let fc = fc.min(SR as f32 * 0.45);
        let w0 = TAU * fc / SR as f32;
        let cw = w0.cos();
        let al = w0.sin() / (2.0 * q);
        let (b0, b1, b2) = match ft {
            Ft::Low => ((1.0 - cw) / 2.0, 1.0 - cw, (1.0 - cw) / 2.0),
            Ft::High => ((1.0 + cw) / 2.0, -(1.0 + cw), (1.0 + cw) / 2.0),
            Ft::Band => (al, 0.0, -al),
        };
        let a0 = 1.0 + al;
        [b0 / a0, b1 / a0, b2 / a0, -2.0 * cw / a0, (1.0 - al) / a0]
    }
    fn biquad(x: &[f32], ft: Ft, fc: f32, q: f32) -> Vec<f32> {
        let c = coef(ft, fc, q);
        let (mut x1, mut x2, mut y1, mut y2) = (0.0, 0.0, 0.0, 0.0);
        x.iter().map(|&xi| {
            let yi = c[0] * xi + c[1] * x1 + c[2] * x2 - c[3] * y1 - c[4] * y2;
            x2 = x1;
            x1 = xi;
            y2 = y1;
            y1 = yi;
            yi
        }).collect()
    }
    fn wav(buf: &[f32]) -> Vec<u8> {
        let n = buf.len() as u32;
        let mut o = Vec::with_capacity(44 + buf.len() * 2);
        o.extend_from_slice(b"RIFF");
        o.extend_from_slice(&(36 + n * 2).to_le_bytes());
        o.extend_from_slice(b"WAVEfmt ");
        o.extend_from_slice(&16u32.to_le_bytes());
        o.extend_from_slice(&1u16.to_le_bytes());
        o.extend_from_slice(&1u16.to_le_bytes());
        o.extend_from_slice(&(SR as u32).to_le_bytes());
        o.extend_from_slice(&(SR as u32 * 2).to_le_bytes());
        o.extend_from_slice(&2u16.to_le_bytes());
        o.extend_from_slice(&16u16.to_le_bytes());
        o.extend_from_slice(b"data");
        o.extend_from_slice(&(n * 2).to_le_bytes());
        for s in buf {
            o.extend_from_slice(&((s.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
        }
        o
    }
    async fn snd(buf: &[f32]) -> Option<Sound> {
        load_sound_from_bytes(&wav(buf)).await.ok()
    }

    pub struct Audio {
        sfx: HashMap<&'static str, Sound>,
        lead: HashMap<&'static str, Sound>,
        bass: HashMap<&'static str, Sound>,
        drum: HashMap<u8, Sound>,
        motor: Vec<(Sound, f32)>,
        rain: Option<(Sound, f32)>,
        wind: Option<(Sound, f32)>,
        pub music_on: bool,
        step: usize,
        acc: f32,
    }
    impl Audio {
        pub async fn new(music_on: bool) -> Audio {
            let mut a = Audio { sfx: HashMap::new(), lead: HashMap::new(), bass: HashMap::new(), drum: HashMap::new(), motor: vec![], rain: None, wind: None, music_on, step: 0, acc: 0.0 };
            let mut seed = 0x1234_5678u32;
            let mut mk = |dur: f32, f: &dyn Fn(&mut Gen)| {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223) | 1;
                let mut g = Gen::new(dur, seed);
                f(&mut g);
                g.buf
            };
            let defs: Vec<(&'static str, Vec<f32>)> = vec![
                ("jump", mk(0.15, &|g| g.tone(0.0, Wv::Tri, 380.0, 820.0, 0.12, 0.35))),
                ("spring", mk(0.28, &|g| g.tone(0.0, Wv::Square, 260.0, 1300.0, 0.25, 0.12))),
                ("tramp", mk(0.3, &|g| g.arp(0.0, &["C5", "E5", "G5", "C6", "E6"], 0.05, Wv::Tri, 0.28, 0.09))),
                ("shoot", mk(0.1, &|g| g.tone(0.0, Wv::Saw, 1500.0, 280.0, 0.08, 0.09))),
                ("kill", mk(0.32, &|g| {
                    g.tone(0.0, Wv::Sine, 140.0, 40.0, 0.3, 0.6);
                    g.noise(0.0, 0.22, 0.5, Ft::Low, 1200.0, true, 0.7071);
                })),
                ("brk", mk(0.32, &|g| g.noise(0.0, 0.3, 0.55, Ft::Low, 1500.0, true, 0.7071))),
                ("puff", mk(0.15, &|g| g.noise(0.0, 0.12, 0.15, Ft::High, 2000.0, false, 0.7071))),
                ("power", mk(0.25, &|g| g.arp(0.0, &["G4", "C5", "E5", "G5", "C6"], 0.04, Wv::Tri, 0.28, 0.09))),
                ("hurt", mk(0.48, &|g| g.tone(0.0, Wv::Square, 700.0, 90.0, 0.45, 0.14))),
                ("fall", mk(0.93, &|g| g.tone(0.0, Wv::Tri, 900.0, 80.0, 0.9, 0.3))),
                ("click", mk(0.08, &|g| g.tone(0.0, Wv::Tri, nf("G5"), nf("G5"), 0.06, 0.25))),
                ("note", mk(0.3, &|g| g.arp(0.0, &["E5", "C5", "G5"], 0.08, Wv::Tri, 0.28, 0.09))),
                ("thunder", mk(1.85, &|g| {
                    g.noise(0.0, 0.12, 0.45, Ft::High, 1500.0, false, 0.7071);
                    g.noise(0.0, 1.8, 0.9, Ft::Low, 350.0, true, 0.7071);
                    g.tone(0.0, Wv::Saw, 1800.0, 90.0, 0.18, 0.08);
                })),
                ("warn", mk(0.17, &|g| {
                    g.tone(0.0, Wv::Square, 1200.0, 1200.0, 0.06, 0.06);
                    g.tone(0.1, Wv::Square, 1200.0, 1200.0, 0.06, 0.06);
                })),
                ("whoosh", mk(0.62, &|g| g.noise(0.0, 0.6, 0.18, Ft::Band, 700.0, false, 1.0))),
                ("hiss", mk(0.1, &|g| g.noise(0.0, 0.08, 0.05, Ft::High, 4000.0, false, 0.7071))),
                ("coin", mk(0.22, &|g| g.arp(0.0, &["B5", "E6"], 0.06, Wv::Tri, 0.28, 0.09))),
                ("rumble", mk(1.25, &|g| g.noise(0.0, 1.2, 0.7, Ft::Low, 180.0, true, 0.7071))),
                ("revive", mk(0.7, &|g| g.arp(0.0, &["C5", "E5", "G5", "C6", "E6", "G6"], 0.07, Wv::Sine, 0.25, 0.22))),
                ("magnet", mk(0.45, &|g| {
                    g.tone(0.0, Wv::Sine, 300.0, 900.0, 0.35, 0.25);
                    g.tone(0.05, Wv::Tri, 600.0, 1200.0, 0.3, 0.12);
                })),
                ("zip", mk(0.12, &|g| g.tone(0.0, Wv::Sine, 500.0, 1500.0, 0.08, 0.15))),
                ("shbreak", mk(0.35, &|g| {
                    g.noise(0.0, 0.25, 0.35, Ft::High, 3500.0, false, 0.7071);
                    g.tone(0.0, Wv::Sine, 1800.0, 500.0, 0.2, 0.2);
                })),
                ("restore", mk(0.35, &|g| g.arp(0.0, &["G4", "B4", "D5", "G5"], 0.06, Wv::Tri, 0.28, 0.12))),
            ];
            for (k, b) in defs {
                if let Some(s) = snd(&b).await {
                    a.sfx.insert(k, s);
                }
            }
            // bucles: motor del jetpack / hélice, lluvia y viento
            let mut g = Gen::new(0.0, 0x9e37_79b9);
            for (fc, base, depth) in [(650.0, 0.92, 0.08), (2600.0, 0.55, 0.45)] {
                let src: Vec<f32> = (0..SR).map(|_| g.rnd()).collect();
                let mut out = biquad(&src, Ft::Band, fc, 1.2);
                for (i, o) in out.iter_mut().enumerate() {
                    *o *= (base + depth * (TAU * 18.0 * i as f32 / SR as f32).sin()) * 2.5;
                }
                if let Some(s) = snd(&out).await {
                    play_sound(&s, PlaySoundParams { looped: true, volume: 0.0 });
                    a.motor.push((s, 0.0));
                }
            }
            let ws: Vec<f32> = (0..SR * 3).map(|_| g.rnd()).collect();
            let rn: Vec<f32> = biquad(&biquad(&ws, Ft::High, 900.0, 0.7071), Ft::Band, 2600.0, 0.6).iter().map(|v| v * 2.0).collect();
            if let Some(s) = snd(&rn).await {
                play_sound(&s, PlaySoundParams { looped: true, volume: 0.0 });
                a.rain = Some((s, 0.0));
            }
            let nw = (SR as f32 / 0.23) as usize;
            let mut l = 0.0;
            let br: Vec<f32> = (0..nw).map(|_| {
                l = (l + 0.02 * g.rnd()) / 1.02;
                l * 3.5
            }).collect();
            let (mut x1, mut x2, mut y1, mut y2) = (0.0, 0.0, 0.0, 0.0);
            let mut c = coef(Ft::Low, 500.0, 1.6);
            let wy: Vec<f32> = (0..nw).map(|i| {
                if i % 64 == 0 {
                    c = coef(Ft::Low, 500.0 + 300.0 * (TAU * 0.23 * i as f32 / SR as f32).sin(), 1.6);
                }
                let yi = c[0] * br[i] + c[1] * x1 + c[2] * x2 - c[3] * y1 - c[4] * y2;
                x2 = x1;
                x1 = br[i];
                y2 = y1;
                y1 = yi;
                yi * 2.0
            }).collect();
            if let Some(s) = snd(&wy).await {
                play_sound(&s, PlaySoundParams { looped: true, volume: 0.0 });
                a.wind = Some((s, 0.0));
            }
            // música: melodía (con eco), bajo y batería
            for nm in MEL.iter().filter(|n| !n.is_empty()) {
                if a.lead.contains_key(nm) {
                    continue;
                }
                let mut g = Gen::new(0.26, 1);
                let f = nf(nm);
                g.tone(0.0, Wv::Square, f, f, 0.233, 0.07);
                let dry = biquad(&g.buf, Ft::Low, 3200.0, 0.7071);
                let d = (0.39 * SR as f32) as usize;
                let mut wet = vec![0.0; dry.len() + d * 4];
                for (i, v) in dry.iter().enumerate() {
                    wet[i] += v;
                    let mut lv = 0.2;
                    for k in 1..=4 {
                        wet[i + d * k] += v * lv;
                        lv *= 0.25;
                    }
                }
                if let Some(s) = snd(&wet).await {
                    a.lead.insert(nm, s);
                }
            }
            for nm in BAS.iter().filter(|n| !n.is_empty()) {
                if a.bass.contains_key(nm) {
                    continue;
                }
                let mut g = Gen::new(0.26, 1);
                let f = nf(nm);
                g.tone(0.0, Wv::Tri, f, f, 0.245, 0.4);
                if let Some(s) = snd(&g.buf).await {
                    a.bass.insert(nm, s);
                }
            }
            let mut k = Gen::new(0.26, 7);
            k.tone(0.0, Wv::Sine, 150.0, 40.0, 0.25, 0.55);
            let mut sn = Gen::new(0.18, 11);
            sn.noise(0.0, 0.15, 0.18, Ft::High, 1500.0, false, 0.7071);
            let mut hh = Gen::new(0.06, 13);
            hh.noise(0.0, 0.04, 0.07, Ft::High, 7000.0, false, 0.7071);
            for (id, b) in [(b'k', k.buf), (b's', sn.buf), (b'h', hh.buf)] {
                if let Some(s) = snd(&b).await {
                    a.drum.insert(id, s);
                }
            }
            a
        }
        pub fn play(&self, n: &str) {
            if let Some(s) = self.sfx.get(n) {
                play_sound(s, PlaySoundParams { looped: false, volume: SFXVOL });
            }
        }
        // motor: 0 nada, 1 jetpack, 2 hélice
        pub fn update(&mut self, dt: f32, motor: u8, rain: f32, wind: f32, bpm: f32) {
            for (i, (s, gain)) in self.motor.iter_mut().enumerate() {
                let tgt = if motor as usize == i + 1 { if i == 0 { 0.6 } else { 0.3 } } else { 0.0 };
                *gain += (tgt - *gain) * (dt * if tgt > 0.0 { 30.0 } else { 15.0 }).min(1.0);
                set_sound_volume(s, gain.max(0.0) * SFXVOL);
            }
            if let Some((s, gn)) = &mut self.rain {
                *gn += (rain - *gn) * (dt * 2.5).min(1.0);
                set_sound_volume(s, gn.max(0.0) * SFXVOL);
            }
            if let Some((s, gn)) = &mut self.wind {
                *gn += (wind - *gn) * (dt * 2.0).min(1.0);
                set_sound_volume(s, gn.max(0.0) * SFXVOL);
            }
            if !self.music_on {
                return;
            }
            let sp = 30.0 / bpm;
            self.acc += dt;
            while self.acc >= sp {
                self.acc -= sp;
                let i = self.step % 32;
                let p = || PlaySoundParams { looped: false, volume: MUSVOL };
                if let Some(s) = self.lead.get(MEL[i]) {
                    play_sound(s, p());
                }
                if let Some(s) = self.bass.get(BAS[i]) {
                    play_sound(s, p());
                }
                if let Some(s) = self.drum.get(&DRM[i % 8]) {
                    play_sound(s, p());
                }
                self.step += 1;
            }
        }
    }
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
    fn dash_circle(&self, cx: f32, cy: f32, r: f32, lw: f32, c: &str) {
        let mut a = 0.0;
        while a < TAU {
            let p = arc_pts(cx, cy, r, a, a + 5.0 / r, 3);
            self.stroke(&p, lw, c, false);
            a += 10.0 / r;
        }
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
struct Plat { x: f32, y: f32, w: f32, t: PT, vx: f32, item: Item, ix: f32, used: f32, broken: bool, vy: f32, gone: bool, rot: f32, x0: f32, rng: f32,
    snow: f32, cor: f32, burnt: bool, tgt: f32, pop: f32, sw: f32 }
#[derive(Clone)]
struct Mon { x: f32, y: f32, bx: f32, bat: bool, w: f32, h: f32, ph: f32, dead: bool, vy: f32, fvx: f32 }
struct Coin { x: f32, y: f32, ph: f32, got: bool, vy: f32, mag: bool }
struct Bullet { x: f32, y: f32 }
struct Part { x: f32, y: f32, vx: f32, vy: f32, life: f32, max: f32, size: f32, col: Color, kind: u8, rot: f32, vr: f32, grav: f32 }
struct Drop { x: f32, y: f32, vx: f32, vy: f32, snow: bool, acid: bool, ph: f32, dead: bool }
struct Rock { x: f32, y: f32, vx: f32, vy: f32, r: f32, pts: Vec<f32>, rot: f32, dead: bool, hail: bool }
struct Bolt { x: f32, y: f32, life: f32, mx: f32 }
struct WxState { ty: Option<Wx>, t: f32, dur: f32, inten: f32, fade: f32, next: f32, last: Option<Wx>, ev: f32, wind: f32, wdir: f32 }
#[derive(Clone, Default)]
struct Look { dudu: bool, hat: String, acc: String, col: String }
struct Ring { x: f32, y: f32, t: f32, col: Color }
struct FTxt { x: f32, y: f32, t: f32, s: String }
struct Chain { y: f32, x: f32, t: PT, l0: f32, l1: f32 }

#[derive(Default)]
struct Player { x: f32, y: f32, vx: f32, vy: f32, face: f32, pu: Item, put: f32, shoot_t: f32, sq: f32, flip: f32, dead: bool,
    shield: bool, imm: f32, rev: f32, rev_from: f32, rev_to: f32, mag: f32, wing: f32 }

#[derive(PartialEq, Clone, Copy)]
enum State { Menu, Play, Pause, Over, Shop, Daily, Ach, Opts, Conf }

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

struct Save { coins: i64, dudu: bool, mode: usize, dif: usize, bests: Vec<(String, i64)>, own: Vec<String>, eq: Vec<(String, String)>, pw: Vec<String>,
    st: Vec<(String, i64)>, ach: Vec<String>, dday: i64, mis: Vec<Mis>, streak: i64, sday: i64, chest_day: i64, music: bool, mode_list: Vec<String> }
impl Save {
    fn load() -> Save {
        let mut s = Save { coins: 0, dudu: false, mode: 0, dif: 1, bests: vec![], own: vec![], eq: vec![], pw: vec![],
            st: vec![], ach: vec![], dday: -1, mis: vec![], streak: 0, sday: -1, chest_day: -1, music: true, mode_list: vec![] };
        if let Ok(txt) = std::fs::read_to_string(save_path()) {
            for l in txt.lines() {
                if let Some((k, v)) = l.split_once('=') {
                    if k == "own" {
                        s.own = v.split(',').filter(|x| !x.is_empty()).map(String::from).collect();
                        continue;
                    }
                    if k == "pw" {
                        s.pw = v.split(',').filter(|x| !x.is_empty()).map(String::from).collect();
                        continue;
                    }
                    let list = |v: &str| v.split(',').filter(|x| !x.is_empty()).map(String::from).collect::<Vec<_>>();
                    match k {
                        "ach" => { s.ach = list(v); continue; }
                        "modes_list" => { s.mode_list = list(v); continue; }
                        "mis" => {
                            s.mis = v.split(';').filter_map(|e| {
                                let f: Vec<&str> = e.split(':').collect();
                                if f.len() != 5 { return None; }
                                Some(Mis { k: f[0].into(), n: f[1].parse().ok()?, p: f[2].parse().ok()?, r: f[3].parse().ok()?, c: f[4] == "1" })
                            }).collect();
                            continue;
                        }
                        _ => {}
                    }
                    if let Some(c) = k.strip_prefix("st_") {
                        s.st.push((c.to_string(), v.trim().parse().unwrap_or(0)));
                        continue;
                    }
                    if let Some(c) = k.strip_prefix("eq_") {
                        if !v.is_empty() {
                            s.eq.push((c.to_string(), v.to_string()));
                        }
                        continue;
                    }
                    let n: i64 = v.trim().parse().unwrap_or(0);
                    match k {
                        "coins" => s.coins = n,
                        "dudu" => s.dudu = n == 1,
                        "mode" => s.mode = (n as usize).min(MODES.len() - 1),
                        "dif" => s.dif = (n as usize).min(2),
                        "dday" => s.dday = n,
                        "streak" => s.streak = n,
                        "sday" => s.sday = n,
                        "chest" => s.chest_day = n,
                        "music" => s.music = n == 1,
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
        o += &format!("own={}\npw={}\n", self.own.join(","), self.pw.join(","));
        o += &format!("ach={}\nmodes_list={}\ndday={}\nstreak={}\nsday={}\nchest={}\nmusic={}\n", self.ach.join(","), self.mode_list.join(","), self.dday, self.streak, self.sday, self.chest_day, self.music as i32);
        let ms: Vec<String> = self.mis.iter().map(|m| format!("{}:{}:{}:{}:{}", m.k, m.n, m.p, m.r, m.c as i32)).collect();
        o += &format!("mis={}\n", ms.join(";"));
        for (k, v) in &self.st {
            o += &format!("st_{}={}\n", k, v);
        }
        for (k, v) in &self.eq {
            o += &format!("eq_{}={}\n", k, v);
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
    fn stat(&self, k: &str) -> i64 {
        self.st.iter().find(|(a, _)| a == k).map(|(_, v)| *v).unwrap_or(0)
    }
    fn stat_set(&mut self, k: &str, v: i64) {
        if let Some(e) = self.st.iter_mut().find(|(a, _)| a == k) {
            e.1 = v;
        } else {
            self.st.push((k.to_string(), v));
        }
    }
    fn stat_add(&mut self, k: &str, n: i64) {
        let v = self.stat(k) + n;
        self.stat_set(k, v);
    }
    fn stat_max(&mut self, k: &str, v: i64) {
        if v > self.stat(k) {
            self.stat_set(k, v);
        }
    }
    fn eq_get(&self, c: &str) -> String {
        self.eq.iter().find(|(a, _)| a == c).map(|(_, v)| v.clone()).unwrap_or_default()
    }
    fn eq_set(&mut self, c: &str, v: &str) {
        self.eq.retain(|(a, _)| a != c);
        if !v.is_empty() {
            self.eq.push((c.to_string(), v.to_string()));
        }
    }
    fn has_pw(&self, id: &str) -> bool {
        self.pw.iter().any(|x| x == id)
    }
    fn use_pw(&mut self, id: &str) {
        self.pw.retain(|x| x != id);
    }
    fn look(&self) -> Look {
        Look { dudu: self.dudu, hat: self.eq_get("hat"), acc: self.eq_get("acc"), col: self.eq_get("col") }
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
    wx: WxState,
    bag: Vec<Wx>,
    drops: Vec<Drop>,
    rocks: Vec<Rock>,
    warns: Vec<(f32, f32)>,
    bolts: Vec<Bolt>,
    note: Option<(String, f32)>,
    gm: f32,
    lost: Vec<(f32, f32, PT)>,
    revives: i64,
    safe_x: f32,
    safe_y: f32,
    shop_tab: usize,
    shop_sel: String,
    toast: (String, f32),
    night_noted: bool,
    banners: Vec<(String, u8, f32)>,
    chest_t: f32,
    chest_prize: (String, String, i64),
    ach_t: f32,
    opt_back: State,
    sfx_q: Vec<&'static str>,
    mag_snd_t: f32,
    fullscreen: bool,
}

impl Game {
    fn new() -> Game {
        let mut g = Game {
            w: BASE_W, h: BASE_H, state: State::Menu, p: Player::default(), plats: vec![], mons: vec![], coins: vec![], bullets: vec![],
            parts: vec![], rings: vec![], ftxt: vec![], chains: vec![], safe_tab: vec![], score: 0.0, cam_off: 0.0, shake: 0.0, flash: 0.0,
            time: 0.0, over_t: 0.0, tl: TIME_LIMIT, combo: 0, combo_y: 1e9, combo_pop: 0.0, run_max_combo: 0, run_coins: 0, new_best: false,
            fire_t: 0.0, swap_t: 0.0, sv: Save::load(), btns: vec![], touch_dir: 0.0, touch_fire: false,
            touch_ui: cfg!(target_os = "android"), zones: [false; 3],
            wx: WxState { ty: None, t: 0.0, dur: 0.0, inten: 0.0, fade: 0.0, next: 5.0, last: None, ev: 0.0, wind: 0.0, wdir: 1.0 },
            bag: vec![], drops: vec![], rocks: vec![], warns: vec![], bolts: vec![], note: None, gm: 1.0, lost: vec![], revives: 0,
            safe_x: BASE_W / 2.0, safe_y: BASE_H - 60.0, shop_tab: 0, shop_sel: String::new(), toast: (String::new(), 0.0), night_noted: false,
            banners: vec![], chest_t: -1.0, chest_prize: (String::new(), String::new(), 0), ach_t: 0.0, opt_back: State::Menu, sfx_q: vec![],
            mag_snd_t: 0.0, fullscreen: false,
        };
        g.refresh_day();
        g.build_safe_tab();
        g
    }
    fn dif(&self) -> (f32, f32, f32) {
        let d = DIFS[self.sv.dif];
        (d.1, d.2, d.3)
    }
    fn dif_w(&self) -> f32 {
        DIFS[self.sv.dif].4
    }
    fn dif_i(&self) -> f32 {
        DIFS[self.sv.dif].5
    }
    fn chaos(&self) -> bool {
        MODES[self.sv.mode].0 == "chaos"
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
        Plat { x, y, w: 58.0, t, vx, item: Item::None, ix: 0.0, used: 0.0, broken: false, vy: 0.0, gone: false, rot: 0.0, x0: x, rng: 45.0,
            snow: 0.0, cor: 0.0, burnt: false, tgt: 0.0, pop: 0.0, sw: rnd(0.0, 6.0) }
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
            let mut guard = 0;
            while self.chains[i].y > -60.0 && guard < 60 {
                self.gen_one(i);
                guard += 1;
            }
        }
    }
    // cambio de tamaño a mitad de partida: conserva lo ya generado
    fn resize_chains(&mut self) {
        let n = if self.w > 950.0 { 3 } else if self.w > 600.0 { 2 } else { 1 };
        let lw = self.w / n as f32;
        let top = self.chains.iter().map(|c| c.y).fold(self.h - 60.0, f32::min);
        let mut out = vec![];
        for i in 0..n {
            let (l0, l1) = (i as f32 * lw, (i + 1) as f32 * lw);
            let best = self.plats.iter()
                .filter(|p| !p.broken && !p.gone && p.x + 29.0 >= l0 && p.x + 29.0 < l1)
                .min_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
            let (y, x, t) = match best {
                Some(p) => (p.y, p.x.clamp(l0, (l1 - 58.0).max(l0)), p.t),
                None => (top, l0 + lw / 2.0 - 29.0, PT::G),
            };
            out.push(Chain { y, x, t, l0, l1 });
        }
        self.chains = out;
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
                self.mons.push(Mon { x: mxp, y: my, bx: 0.0, bat: !big, w, h, ph: rnd(0.0, 6.0), dead: false, vy: 0.0, fvx: 0.0 });
            }
        }
        if item == Item::None && t != PT::B && chance() < 0.4 * dc {
            let n = if chance() < 0.25 { 3 } else { 1 };
            let (cy2, cx2) = (y - 34.0, x + 29.0);
            if !self.mons.iter().any(|m| !m.dead && (m.y - cy2).abs() < 50.0 && (m.x - cx2).abs() < 90.0) {
                for i in 0..n {
                    self.coins.push(Coin { x: cx2 + (i as f32 - (n - 1) as f32 / 2.0) * 20.0, y: cy2, ph: rnd(0.0, 6.0), got: false, vy: 0.0, mag: false });
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
        self.revives = 0;
        self.night_noted = false;
        self.p.shield = self.sv.has_pw("shield");
        self.safe_x = self.p.x;
        self.safe_y = self.p.y;
        self.reset_wx();
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
    fn fx(&mut self, x: f32, y: f32, n: usize, cols: &[&str], sp: f32, grav: f32, life: f32, size: f32, kind: u8, up: f32) {
        for _ in 0..n {
            let a = rnd(0.0, TAU);
            let s = rnd(sp * 0.4, sp);
            let l = life * rnd(0.7, 1.1);
            self.parts.push(Part {
                x, y, vx: a.cos() * s, vy: a.sin() * s + up, life: l, max: l,
                size: (if kind == K_STAR { 6.0 } else { 3.5 }) * size * rnd(0.7, 1.2), col: hx(pick(cols)), kind, rot: rnd(0.0, TAU), vr: rnd(-6.0, 6.0), grav,
            });
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn add_part(&mut self, x: f32, y: f32, vx: f32, vy: f32, mx: f32, r: f32, col: &str, kind: u8, grav: f32, rot: f32, vr: f32) {
        if self.parts.len() > 260 {
            return;
        }
        self.parts.push(Part { x, y, vx, vy, life: mx, max: mx, size: r, col: hx(col), kind, rot, vr, grav });
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
            p.rot += dt * p.vr;
        }
        self.parts.retain(|p| p.life > 0.0);
        if self.parts.len() > 420 {
            let k = self.parts.len() - 420;
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
        let c = self.combo as i64;
        self.sv.stat_max("combo_best", c);
        self.mis_max("combo", c);
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



    // ---------- sonido ----------
    fn snd(&mut self, n: &'static str) {
        if self.sfx_q.len() < 24 {
            self.sfx_q.push(n);
        }
    }

    // ---------- diario, misiones, cofre, logros ----------
    fn gen_missions(d: i64) -> Vec<Mis> {
        let mut seed = (d as u64).wrapping_mul(7919).wrapping_add(13);
        let mut r = |n: usize| {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((seed >> 33) as usize) % n
        };
        let mut idx: Vec<usize> = (0..MISSIONS.len()).collect();
        for i in (1..idx.len()).rev() {
            let j = r(i + 1);
            idx.swap(i, j);
        }
        (0..3).map(|i| {
            let m = MISSIONS[idx[i]];
            let ti = r(m.2.len());
            Mis { k: m.0.to_string(), n: m.2[ti], p: 0, r: m.3 + ti as i64 * 10, c: false }
        }).collect()
    }
    fn refresh_day(&mut self) {
        let d = today();
        if self.sv.dday != d {
            self.sv.dday = d;
            self.sv.mis = Self::gen_missions(d);
            self.sv.write();
        }
        if self.sv.sday != d {
            self.sv.streak = if self.sv.sday == d - 1 { self.sv.streak + 1 } else { 1 };
            self.sv.sday = d;
            let st = self.sv.streak;
            self.sv.stat_max("streak_best", st);
            if st > 1 {
                self.banner(&format!("racha de {} días · ¡abre tu cofre!", st), 2);
            }
            self.check_ach();
            self.sv.write();
        }
    }
    fn banner(&mut self, t: &str, kind: u8) {
        if self.banners.len() < 5 {
            self.banners.push((t.to_string(), kind, -1.0));
        }
    }
    fn update_banners(&mut self, dt: f32) {
        let Some(b) = self.banners.first_mut() else { return };
        if b.2 < 0.0 {
            b.2 = 0.0;
            let k = b.1;
            self.snd(if k == 1 { "power" } else { "coin" });
        }
        let b = &mut self.banners[0];
        b.2 += dt;
        if b.2 > 2.8 {
            self.banners.remove(0);
        }
    }
    fn mis_add(&mut self, k: &str, n: i64) {
        let mut done = false;
        for m in &mut self.sv.mis {
            if m.k == k && m.p < m.n {
                m.p = (m.p + n).min(m.n);
                done |= m.p >= m.n;
            }
        }
        if done {
            self.banner("¡misión cumplida! cóbrala en el diario", 0);
            self.sv.write();
        }
    }
    fn mis_max(&mut self, k: &str, v: i64) {
        let mut done = false;
        for m in &mut self.sv.mis {
            if m.k == k && m.p < m.n && v > m.p {
                m.p = v.min(m.n);
                done |= m.p >= m.n;
            }
        }
        if done {
            self.banner("¡misión cumplida! cóbrala en el diario", 0);
            self.sv.write();
        }
    }
    fn claim_mis(&mut self, i: usize) {
        let Some(m) = self.sv.mis.get_mut(i) else { return };
        if m.c || m.p < m.n {
            return;
        }
        m.c = true;
        let r = m.r;
        self.sv.coins += r;
        self.snd("power");
        self.toast(&format!("+{} monedas", r));
        let ox = (self.w - BASE_W) / 2.0;
        self.fx(ox + 300.0, 412.0 + i as f32 * 60.0, 10, &["#f3d23c", "#ffffff"], 140.0, 200.0, 0.7, 1.0, K_STAR, 0.0);
        self.sv.write();
    }
    fn chest_ready(&self) -> bool {
        self.sv.chest_day != today()
    }
    fn daily_pending(&self) -> bool {
        self.chest_ready() || self.sv.mis.iter().any(|m| m.p >= m.n && !m.c)
    }
    fn open_chest(&mut self) {
        if !self.chest_ready() || self.chest_t >= 0.0 {
            return;
        }
        self.sv.chest_day = today();
        self.sv.stat_add("chests", 1);
        let big = self.sv.streak >= 7;
        let r = chance();
        let mut prize: Option<(String, String)> = None;
        if r < if big { 0.25 } else { 0.12 } {
            let mut opts = vec![];
            for c in ["hat", "acc", "col", "trl"] {
                for it in shop_items(c) {
                    if !self.sv.own.contains(&format!("{}:{}", c, it.0)) {
                        opts.push((c, *it));
                    }
                }
            }
            if !opts.is_empty() {
                let o = opts[rand::gen_range(0, opts.len())];
                self.sv.own.push(format!("{}:{}", o.0, o.1 .0));
                prize = Some((format!("¡{}!", o.1 .1), "nuevo en la tienda".into()));
            }
        }
        if prize.is_none() && r < if big { 0.5 } else { 0.35 } {
            let pws: Vec<_> = SHOP_POW.iter().filter(|it| !self.sv.has_pw(it.0)).collect();
            if !pws.is_empty() {
                let it = pws[rand::gen_range(0, pws.len())];
                self.sv.pw.push(it.0.to_string());
                prize = Some((format!("¡{}!", it.1), "poder listo para usar".into()));
            }
        }
        let prize = prize.unwrap_or_else(|| {
            let n = rand::gen_range(15, 41) * if big { 2 } else { 1 };
            self.sv.coins += n;
            (format!("+{} monedas", n), String::new())
        });
        let bonus = STREAK_R[(self.sv.streak.clamp(1, 7) - 1) as usize];
        self.sv.coins += bonus;
        self.chest_prize = (prize.0, prize.1, bonus);
        self.chest_t = 0.0;
        self.snd("power");
        self.check_ach();
        self.sv.write();
    }
    fn check_ach(&mut self) {
        for a in ACHS {
            if self.sv.ach.iter().any(|x| x == a.0) {
                continue;
            }
            if self.sv.stat(a.3) >= a.4 {
                self.sv.ach.push(a.0.to_string());
                self.sv.coins += a.5;
                self.banner(&format!("logro: {}  +{}", a.1, a.5), 1);
                self.sv.write();
            }
        }
    }
    fn track_alt(&mut self) {
        let a = self.alt_m();
        self.sv.stat_max("alt_best", a);
        self.mis_max("alt", a);
        if self.timed() {
            self.sv.stat_max("time_alt_best", a);
        }
    }
    fn reset_progress(&mut self) {
        let music = self.sv.music;
        self.sv = Save { coins: 0, dudu: false, mode: 0, dif: 1, bests: vec![], own: vec![], eq: vec![], pw: vec![],
            st: vec![], ach: vec![], dday: -1, mis: vec![], streak: 0, sday: -1, chest_day: -1, music, mode_list: vec![] };
        self.refresh_day();
        self.sv.write();
        self.state = State::Opts;
        self.toast("progreso borrado");
    }

    // ---------- clima / eventos (update_wx de game.gd) ----------
    fn reset_wx(&mut self) {
        let next = if self.chaos() { 1.0 } else { rnd(3.0, 6.0) * self.dif_w() };
        self.wx = WxState { ty: None, t: 0.0, dur: 0.0, inten: 0.0, fade: 0.0, next, last: None, ev: 0.0, wind: 0.0, wdir: 1.0 };
        self.bag.clear();
        self.drops.clear();
        self.rocks.clear();
        self.warns.clear();
        self.bolts.clear();
        self.lost.clear();
        self.note = None;
        self.gm = 1.0;
    }
    fn next_wx(&mut self) -> Wx {
        if self.bag.is_empty() {
            let zen = self.zen();
            self.bag = WX_ALL.iter().copied().filter(|k| !zen || WX_ZEN.contains(k)).collect();
            for i in (1..self.bag.len()).rev() {
                let j = rand::gen_range(0, i + 1);
                self.bag.swap(i, j);
            }
            if self.bag.last().copied() == self.wx.last {
                let l = self.bag.pop().unwrap();
                self.bag.insert(0, l);
            }
        }
        self.bag.pop().unwrap()
    }
    fn show_note(&mut self, t: &str) {
        self.note = Some((t.to_string(), 0.0));
        self.snd("note");
    }
    fn start_wx(&mut self) {
        let tt = self.next_wx();
        self.wx.ty = Some(tt);
        self.wx.last = Some(tt);
        self.wx.t = 0.0;
        self.lost.clear();
        self.wx.dur = 8.0 + (self.score / 1000.0).min(14.0);
        self.wx.inten = ((0.45 + self.score / 15000.0) * self.dif_i() * if self.chaos() { 1.25 } else { 1.0 }).min(1.0);
        self.wx.ev = 1.2;
        self.wx.wdir = if chance() < 0.5 { -1.0 } else { 1.0 };
        self.show_note(&format!("¡{}!", wx_name(tt)));
    }
    fn stable(q: &Plat) -> bool {
        !q.broken && !q.gone && !q.burnt && q.t != PT::K
    }
    fn reach_ok(&self, a: &Plat, b: &Plat) -> bool {
        let dv = a.y - b.y;
        if dv <= 0.0 || dv > REACH {
            return false;
        }
        let slack = if a.t == PT::B { B_SLACK } else { 0.0 } + if b.t == PT::B { B_SLACK } else { 0.0 };
        let px = |p: &Plat| if p.t == PT::B { p.x0 } else { p.x };
        self.wdist(px(a), px(b)) <= self.safe_dx(dv) - slack
    }
    // ¿se puede romper p sin dejar a otra plataforma sin camino hacia arriba?
    fn can_lose(&self, pi: usize) -> bool {
        let p = &self.plats[pi];
        if p.t == PT::K || p.burnt {
            return true;
        }
        for (qi, q) in self.plats.iter().enumerate() {
            if qi == pi || !Self::stable(q) {
                continue;
            }
            let mut has_p = false;
            let mut other = false;
            for (ri, r) in self.plats.iter().enumerate() {
                if r.y < q.y && Self::stable(r) && self.reach_ok(q, r) {
                    if ri == pi {
                        has_p = true;
                    } else {
                        other = true;
                    }
                }
            }
            if has_p && !other {
                return false;
            }
        }
        true
    }
    fn break_plat(&mut self, i: usize, ev: bool) {
        if self.plats[i].broken {
            return;
        }
        let (x, y, t, burnt) = (self.plats[i].x, self.plats[i].y, self.plats[i].t, self.plats[i].burnt);
        if ev || (burnt && self.wx.ty.is_some()) {
            self.lost.push((x, y, if burnt { PT::G } else { t }));
        }
        let p = &mut self.plats[i];
        p.broken = true;
        p.vy = 0.0;
        p.item = Item::None;
        p.tgt = 0.0;
        self.snd("brk");
        let cols: &[&str] = if burnt { &["#4a3a33", "#2b2420", "#f39b2d"] } else { &["#b8865a", "#8a5e3a", "#d9b48a"] };
        self.fx(x + 29.0, y + 7.0, 10, cols, 120.0, 900.0, 0.8, 1.1, K_DOT, 0.0);
    }
    fn strike(&mut self, i: usize) {
        self.plats[i].burnt = true;
        self.plats[i].item = Item::None;
        let (bx, by) = (self.plats[i].x + self.plats[i].w / 2.0, self.plats[i].y);
        self.bolts.push(Bolt { x: bx, y: by, life: 0.0, mx: 0.28 });
        self.snd("thunder");
        self.flash = self.flash.max(0.9);
        self.shake = self.shake.max(7.0);
        self.fx(bx, by, 12, &["#f3d23c", "#ffffff"], 200.0, 300.0, 0.6, 0.8, K_STAR, 0.0);
        self.ring(bx, by, "#f3d23c");
        if !self.p.dead && (self.p.x - bx).hypot((self.p.y - 20.0) - by) < 85.0 {
            self.p.vy = -980.0;
            self.p.flip = 1.0;
            self.p.sq = 1.0;
            let (x, y) = (self.p.x, self.p.y - 60.0);
            self.float_txt(x, y, "¡zas!");
        }
    }
    fn spawn_drop(&mut self, kind: u8) {
        let w = self.w;
        if kind == 2 {
            self.drops.push(Drop { x: rnd(-20.0, w + 20.0), y: -10.0, vx: rnd(-20.0, 20.0), vy: rnd(50.0, 110.0), snow: true, acid: false, ph: rnd(0.0, 6.0), dead: false });
        } else {
            self.drops.push(Drop { x: rnd(-20.0, w + 40.0), y: rnd(-30.0, -5.0), vx: -40.0, vy: rnd(620.0, 820.0), snow: false, acid: kind == 1, ph: 0.0, dead: false });
        }
    }
    fn spawn_rate(rate: f32) -> usize {
        let mut n = rate;
        let mut c = 0;
        while n > 1.0 {
            c += 1;
            n -= 1.0;
        }
        if chance() < n {
            c += 1;
        }
        c
    }
    fn kill_rock(&mut self, i: usize) {
        let (x, y, hail) = (self.rocks[i].x, self.rocks[i].y, self.rocks[i].hail);
        self.rocks[i].dead = true;
        self.score += if hail { 5.0 } else { 25.0 };
        self.shake = self.shake.max(4.0);
        self.fx(x, y, 10, &["#8a7466", "#5e4c42", "#f39b2d"], 180.0, 600.0, 0.8, 1.2, K_DOT, 0.0);
        self.float_txt(x, y - 24.0, if hail { "+5" } else { "+25" });
    }
    fn end_event(&mut self) {
        if self.lost.is_empty() {
            return;
        }
        if self.sv.has_pw("respawn") {
            let lost = std::mem::take(&mut self.lost);
            let mut n = 0;
            for (x, y, t) in lost {
                if y < -30.0 || y > self.h - 30.0 || !self.free_spot(x, y, 58.0, false) {
                    continue;
                }
                let p = self.mk(x, y, t);
                self.plats.push(p);
                n += 1;
                self.ring(x + 29.0, y + 7.0, "#86d443");
                self.fx(x + 29.0, y + 7.0, 6, &["#86d443", "#ffffff", "#f3d23c"], 100.0, -20.0, 0.6, 0.7, K_STAR, 0.0);
            }
            if n > 0 {
                self.snd("restore");
                self.sv.use_pw("respawn");
                self.sv.write();
                let w = self.w;
                self.float_txt(w / 2.0, 130.0, "¡plataformas de vuelta!");
            }
        }
        self.lost.clear();
    }

    fn update_wx(&mut self, dt: f32) {
        let (w, h) = (self.w, self.h);
        if self.wx.ty.is_none() {
            if self.score >= if self.chaos() { 0.0 } else { 600.0 } {
                self.wx.next -= dt;
                if self.wx.next <= 0.0 {
                    self.start_wx();
                }
            }
        } else {
            self.wx.t += dt;
            if self.wx.t > self.wx.dur {
                self.wx.ty = None;
                self.end_event();
                self.wx.next = rnd(6.0, 11.0) * if self.chaos() { 0.1 } else { self.dif_w() };
            }
        }
        let tt = self.wx.ty;
        let ii = self.wx.inten;
        self.wx.fade = if tt.is_some() { (self.wx.t / 1.5).min((self.wx.dur - self.wx.t) / 1.5).clamp(0.0, 1.0) } else { 0.0 };
        let ff = self.wx.fade;
        if let Some(n) = &mut self.note {
            n.1 += dt;
            if n.1 > 3.4 {
                self.note = None;
            }
        }
        match tt {
            Some(Wx::Rain) | Some(Wx::Storm) | Some(Wx::Acid) => {
                let rate = if tt == Some(Wx::Storm) { 1.5 } else { 1.0 } * ii * ff * 95.0 * dt;
                for _ in 0..Self::spawn_rate(rate) {
                    self.spawn_drop(if tt == Some(Wx::Acid) { 1 } else { 0 });
                }
            }
            Some(Wx::Snow) => {
                for _ in 0..Self::spawn_rate(ii * ff * 45.0 * dt) {
                    self.spawn_drop(2);
                }
            }
            _ => {}
        }
        let wt = if tt == Some(Wx::Wind) { self.wx.wdir * ii * ff } else { 0.0 };
        self.wx.wind += (wt - self.wx.wind) * (dt * 2.0).min(1.0);
        if tt == Some(Wx::Wind) && ff > 0.1 {
            let sx = if self.wx.wdir > 0.0 { -15.0 } else { w + 15.0 };
            let wd = self.wx.wdir;
            for _ in 0..Self::spawn_rate(7.0 * ii * ff * dt) {
                let c = pick(&["#e8a23c", "#c9612f", "#86d443", "#d4b24a"]);
                self.add_part(sx, rnd(40.0, h), wd * rnd(170.0, 260.0), rnd(-30.0, 40.0), 2.6, rnd(3.0, 5.0), c, K_LEAF, 20.0, rnd(0.0, 6.0), rnd(-7.0, 7.0));
            }
            for _ in 0..Self::spawn_rate(4.0 * ii * ff * dt) {
                self.add_part(sx, rnd(40.0, h), wd * rnd(380.0, 480.0), 0.0, 1.3, rnd(2.5, 4.0), "#9b8c80", K_GUST, 0.0, 0.0, 0.0);
            }
        }
        for p in &mut self.plats {
            if tt == Some(Wx::Snow) && ff > 0.2 && p.t != PT::W {
                p.snow = (p.snow + dt * 0.3).min(1.0);
            } else {
                p.snow = (p.snow - dt * 0.12).max(0.0);
            }
        }
        if tt == Some(Wx::Storm) && ff > 0.5 {
            self.wx.ev -= dt;
            if self.wx.ev <= 0.0 {
                self.wx.ev = rnd(1.6, 3.0) / ii.max(0.1);
                let c: Vec<usize> = (0..self.plats.len())
                    .filter(|&i| {
                        let p = &self.plats[i];
                        p.y > 70.0 && p.y < h - 40.0 && !p.broken && !p.gone && p.t != PT::W && !p.burnt && p.tgt <= 0.0
                    })
                    .filter(|&i| self.can_lose(i))
                    .collect();
                if !c.is_empty() {
                    let i = c[rand::gen_range(0, c.len())];
                    self.plats[i].tgt = 1.1;
                }
            }
        }
        if tt == Some(Wx::Meteor) && ff > 0.4 {
            self.wx.ev -= dt;
            if self.wx.ev <= 0.0 {
                self.wx.ev = rnd(0.9, 1.8) / ii.max(0.1);
                self.warns.push((rnd(25.0, w - 25.0), 0.9));
                self.snd("warn");
            }
        }
        let mut fire = vec![];
        for wn in &mut self.warns {
            wn.1 -= dt;
            if wn.1 <= 0.0 {
                fire.push(wn.0);
            }
        }
        self.warns.retain(|wn| wn.1 > 0.0);
        for x in fire {
            let r = rnd(11.0, 16.0);
            let pts = (0..9).map(|_| r * rnd(0.82, 1.1)).collect();
            self.snd("whoosh");
            self.rocks.push(Rock { x, y: -30.0, vx: rnd(-35.0, 35.0), vy: 380.0 + ii * 220.0, r, pts, rot: 0.0, dead: false, hail: false });
        }
        if tt == Some(Wx::Hail) && ff > 0.4 {
            self.wx.ev -= dt;
            if self.wx.ev <= 0.0 {
                self.wx.ev = rnd(0.3, 0.7) / ii.max(0.1);
                let r = rnd(5.0, 8.0);
                self.rocks.push(Rock { x: rnd(10.0, w - 10.0), y: -20.0, vx: rnd(-15.0, 15.0), vy: 520.0 + ii * 200.0, r, pts: vec![r; 8], rot: 0.0, dead: false, hail: true });
            }
        }
        if tt == Some(Wx::Birds) && ff > 0.3 {
            self.wx.ev -= dt;
            if self.wx.ev <= 0.0 {
                self.wx.ev = rnd(2.2, 3.6) / ii.max(0.1);
                let d = if chance() < 0.5 { -1.0 } else { 1.0 };
                let by = rnd(110.0, h - 220.0);
                let sp = rnd(90.0, 130.0);
                for i in 0..3 {
                    let x = if d > 0.0 { -30.0 - i as f32 * 55.0 } else { w + 30.0 + i as f32 * 55.0 };
                    self.mons.push(Mon { x, y: by + (i as f32 - 1.0) * 28.0, bx: 0.0, bat: true, w: 56.0, h: 34.0, ph: rnd(0.0, 6.0), dead: false, vy: 0.0, fvx: d * sp });
                }
            }
        }
        if tt == Some(Wx::CoinRain) && ff > 0.3 {
            self.wx.ev -= dt;
            if self.wx.ev <= 0.0 {
                self.wx.ev = rnd(0.3, 0.55);
                self.coins.push(Coin { x: rnd(20.0, w - 20.0), y: -14.0, ph: rnd(0.0, 6.0), got: false, vy: rnd(90.0, 140.0), mag: false });
            }
        }
        self.gm += (if tt == Some(Wx::Moon) { 0.55 } else { 1.0 } - self.gm) * (dt * 2.0).min(1.0);
        if tt == Some(Wx::Moon) && chance() < dt * 5.0 {
            self.fx(rnd(0.0, w), h + 5.0, 1, &STARC, 10.0, -20.0, 2.0, 0.7, K_STAR, -60.0);
        }
        if tt == Some(Wx::Quake) && ff > 0.3 {
            self.shake = self.shake.max(2.5 * ff);
            let time = self.time;
            for p in &mut self.plats {
                if !p.broken && p.t != PT::B && p.y < h {
                    p.x = (p.x + (time * 8.0 + p.sw).sin() * 60.0 * ff * dt).clamp(2.0, w - p.w - 2.0);
                }
            }
            self.wx.ev -= dt;
            if self.wx.ev <= 0.0 {
                self.wx.ev = 1.4;
                self.snd("rumble");
                self.fx(rnd(20.0, w - 20.0), -5.0, 5, &["#b8865a", "#8a5e3a", "#d9b48a"], 30.0, 500.0, 0.8, 1.0, K_DOT, 0.0);
            }
        }
        // gotas
        let wind = self.wx.wind;
        let mut hits: Vec<(f32, f32, bool, usize)> = vec![];
        for d in &mut self.drops {
            if d.snow {
                d.ph += dt * 2.0;
                d.x += (d.vx + d.ph.sin() * 25.0 + wind * 80.0) * dt;
                d.y += d.vy * dt;
            } else {
                d.x += (d.vx + wind * 140.0) * dt;
                d.y += d.vy * dt;
            }
            if d.y > h + 10.0 {
                d.dead = true;
                continue;
            }
            for (pi, p) in self.plats.iter().enumerate() {
                if p.broken || p.gone {
                    continue;
                }
                if d.x > p.x && d.x < p.x + p.w && d.y > p.y && d.y < p.y + 12.0 {
                    d.dead = true;
                    if !d.snow {
                        hits.push((d.x, p.y, d.acid, pi));
                    }
                    break;
                }
            }
        }
        self.drops.retain(|d| !d.dead);
        for (x, y, acid, pi) in hits {
            if acid {
                self.fx(x, y, 2, &["#6fcf3a", "#b4f06a"], 50.0, 500.0, 0.3, 0.6, K_DOT, -50.0);
                if self.plats[pi].t != PT::W && !self.plats[pi].broken {
                    self.plats[pi].cor += 0.06;
                    if chance() < 0.15 {
                        self.snd("hiss");
                    }
                    if self.plats[pi].cor >= 1.0 {
                        if self.can_lose(pi) {
                            self.break_plat(pi, true);
                        } else {
                            self.plats[pi].cor = 0.9;
                        }
                    }
                }
            } else if chance() < 0.5 {
                self.fx(x, y, 2, &["#3d6fc4", "#7aaee8"], 60.0, 600.0, 0.3, 0.6, K_DOT, -70.0);
            }
        }
        // rocas y granizo
        for ki in 0..self.rocks.len() {
            {
                let k = &mut self.rocks[ki];
                k.x += k.vx * dt;
                k.y += k.vy * dt;
                k.rot += dt * 3.0;
            }
            let (kx, ky, kr, hail) = (self.rocks[ki].x, self.rocks[ki].y, self.rocks[ki].r, self.rocks[ki].hail);
            if !hail && chance() < 0.7 {
                let c = pick(&["#f39b2d", "#e8504a", "#f3d23c"]);
                self.add_part(kx + rnd(-5.0, 5.0), ky - kr, rnd(-15.0, 15.0), -40.0, rnd(0.25, 0.45), rnd(2.0, 4.0), c, K_DOT, 0.0, 0.0, 0.0);
            }
            if !hail {
                for pi in 0..self.plats.len() {
                    let p = &self.plats[pi];
                    if p.broken || p.gone {
                        continue;
                    }
                    if kx > p.x - kr && kx < p.x + p.w + kr && (ky - (p.y + 7.0)).abs() < kr + 7.0 && self.can_lose(pi) {
                        self.break_plat(pi, true);
                        self.shake = self.shake.max(3.0);
                    }
                }
            }
            if !self.p.dead && !self.rocks[ki].dead && (self.p.x - kx).hypot(self.p.y - 26.0 - ky) < kr + 16.0 {
                if self.p.pu != Item::None {
                    self.kill_rock(ki);
                } else if hail {
                    self.rocks[ki].dead = true;
                    self.p.vy = self.p.vy.max(260.0);
                    self.p.sq = 1.0;
                    self.shake = self.shake.max(4.0);
                    let (x, y) = (self.p.x, self.p.y - 60.0);
                    self.float_txt(x, y, "¡ay!");
                } else {
                    self.rocks[ki].dead = true;
                    self.hurt();
                }
            }
            if ky > h + 40.0 {
                self.rocks[ki].dead = true;
            }
        }
        self.rocks.retain(|k| !k.dead);
        for b in &mut self.bolts {
            b.life += dt;
        }
        self.bolts.retain(|b| b.life < b.mx);
    }

    // ---------- poderes, tienda, revivir ----------
    fn toast(&mut self, t: &str) {
        self.toast = (t.to_string(), 1.6);
    }
    fn use_magnet(&mut self) {
        if self.state != State::Play || self.p.mag > 0.0 || self.p.dead || !self.sv.has_pw("magnet") {
            return;
        }
        self.sv.use_pw("magnet");
        self.p.mag = MAG_T;
        self.snd("magnet");
        self.sv.write();
        let (x, y) = (self.p.x, self.p.y - 30.0);
        self.ring(x, y, "#e8504a");
        self.fx(x, y, 10, &["#e8504a", "#6cc3ee", "#ffffff"], 150.0, 100.0, 0.7, 1.0, K_STAR, 0.0);
    }
    fn revive_cost(&self) -> i64 {
        25 * (self.revives + 1)
    }
    fn revive(&mut self) {
        let cost = self.revive_cost();
        if self.sv.coins < cost {
            self.toast("faltan monedas");
            self.snd("hurt");
            return;
        }
        self.sv.coins -= cost;
        self.revives += 1;
        let p = &mut self.p;
        p.dead = false;
        p.pu = Item::None;
        p.vx = 0.0;
        p.vy = 0.0;
        p.flip = 0.0;
        p.x = self.safe_x.clamp(20.0, self.w - 20.0);
        p.rev_from = self.h + 50.0;
        p.rev_to = self.safe_y;
        p.rev = 1.1;
        p.wing = 4.1;
        p.imm = 0.0;
        self.combo = 0;
        self.combo_y = 1e9;
        self.state = State::Play;
        self.flash = 0.6;
        self.snd("revive");
        self.sv.write();
    }
    fn spawn_revive_plat(&mut self) {
        let px = (self.p.x - 29.0).clamp(4.0, self.w - 62.0);
        let py = self.p.y + 16.0;
        let mut pl = self.mk(px, py, PT::G);
        pl.pop = 1.0;
        self.plats.push(pl);
        self.snd("puff");
        self.ring(px + 29.0, py + 7.0, "#ffffff");
        self.fx(px + 29.0, py + 7.0, 14, &["#ffffff", "#eeeeee", "#f3e6d4"], 110.0, -30.0, 0.8, 2.0, K_DOT, 0.0);
    }
    fn shop_act(&mut self) {
        let cat = TABS[self.shop_tab].0;
        let Some(it) = shop_items(cat).iter().find(|i| i.0 == self.shop_sel).copied() else { return };
        if cat == "pow" {
            if self.sv.has_pw(it.0) {
                self.toast("ya lo tienes (límite 1)");
            } else if self.sv.coins < it.2 {
                self.toast("faltan monedas");
                self.snd("hurt");
            } else {
                self.sv.coins -= it.2;
                self.sv.pw.push(it.0.to_string());
                self.toast("¡comprado!");
                self.snd("power");
            }
            self.sv.write();
            return;
        }
        let k = format!("{}:{}", cat, it.0);
        let had = self.sv.own.contains(&k);
        if !had {
            if self.sv.coins < it.2 {
                self.toast("faltan monedas");
                self.snd("hurt");
                return;
            }
            self.sv.coins -= it.2;
            self.sv.own.push(k);
            self.toast("¡comprado!");
            self.snd("power");
        }
        let cur = self.sv.eq_get(cat);
        let v = if had && cur == it.0 { "" } else { it.0 };
        self.sv.eq_set(cat, v);
        self.sv.write();
    }

    fn shoot(&mut self) {
        if self.p.dead || self.p.pu != Item::None {
            return;
        }
        self.bullets.push(Bullet { x: self.p.x, y: self.p.y - 34.0 });
        self.snd("shoot");
        self.p.shoot_t = 0.25;
        let (x, y) = (self.p.x, self.p.y - 40.0);
        self.fx(x, y, 3, &["#ffffff", "#f3d23c"], 50.0, 0.0, 0.3, 1.0, K_DOT, 0.0);
    }
    fn kill_mon(&mut self, i: usize) {
        self.mons[i].dead = true;
        self.sv.stat_add("kills", 1);
        self.mis_add("kills", 1);
        self.snd("kill");
        self.score += 50.0;
        self.shake = 6.0;
        let (x, y) = (self.mons[i].x + self.mons[i].bx, self.mons[i].y);
        self.fx(x, y, 12, &STARC, 220.0, 500.0, 0.9, 1.0, K_STAR, 0.0);
        self.ring(x, y, "#f3d23c");
        self.float_txt(x, y - 30.0, "+50");
    }
    fn hurt(&mut self) {
        if self.p.dead || self.p.rev > 0.0 || self.p.imm > 0.0 {
            return;
        }
        if self.p.shield {
            self.snd("shbreak");
            self.p.shield = false;
            self.p.imm = 1.0;
            self.p.vy = JUMP * 0.6;
            self.sv.use_pw("shield");
            self.sv.write();
            self.shake = 5.0;
            let (x, y) = (self.p.x, self.p.y - 30.0);
            self.ring(x, y, "#9fd6f2");
            self.fx(x, y, 16, &["#9fd6f2", "#ffffff", "#6cb6e0"], 190.0, 250.0, 0.7, 1.2, K_DOT, 0.0);
            self.float_txt(x, y - 40.0, "¡pop!");
            return;
        }
        self.p.dead = true;
        self.snd("hurt");
        self.p.vy = self.p.vy.max(150.0);
        self.combo_break();
        self.shake = 12.0;
        self.flash = 1.0;
        let (x, y) = (self.p.x, self.p.y - 30.0);
        self.fx(x, y, 14, &["#f3d23c", "#ffffff", "#e8504a"], 200.0, 300.0, 0.8, 1.0, K_STAR, 0.0);
    }
    fn game_over(&mut self) {
        self.state = State::Over;
        self.snd("fall");
        if self.revives == 0 {
            self.sv.stat_add("runs", 1);
            self.mis_add("runs", 1);
            let m = MODES[self.sv.mode].0.to_string();
            if !self.sv.mode_list.contains(&m) {
                self.sv.mode_list.push(m);
                let n = self.sv.mode_list.len() as i64;
                self.sv.stat_set("modes", n);
            }
        }
        self.track_alt();
        let rc = self.run_coins;
        self.sv.stat_max("run_coins_best", rc);
        self.check_ach();
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
        let slippery = self.wx.ty == Some(Wx::Snow) && self.wx.fade > 0.2;
        let target = dir * SPEED + self.wx.wind * WIND_V;
        self.p.vx += (target - self.p.vx) * (dt * if slippery { SNOW_K } else { 10.0 }).min(1.0);
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
        let mut rev_done = false;
        if self.p.rev > 0.0 {
            self.p.rev -= dt;
            let rk = 1.0 - self.p.rev.max(0.0) / 1.1;
            let rk = 1.0 - (1.0 - rk).powi(3);
            self.p.y = self.p.rev_from + (self.p.rev_to - self.p.rev_from) * rk;
            self.p.vy = 0.0;
            if chance() < dt * 40.0 {
                let (x, y) = (self.p.x + rnd(-14.0, 14.0), self.p.y - rnd(0.0, 50.0));
                let c = pick(&["#ffffff", "#fff3a0"]);
                self.add_part(x, y, rnd(-20.0, 20.0), rnd(20.0, 60.0), rnd(0.5, 0.9), rnd(2.0, 3.5), c, K_DOT, 0.0, 0.0, 0.0);
            }
            if self.p.rev <= 0.0 {
                self.p.rev = 0.0;
                self.p.imm = 3.0;
                self.p.vy = JUMP;
                rev_done = true;
            }
        } else if self.p.pu != Item::None {
            self.p.put -= dt;
            self.p.vy = if self.p.pu == Item::Jet { (self.p.vy - 2400.0 * dt).max(-980.0) } else { -640.0 };
            if self.p.put <= 0.0 {
                self.p.pu = Item::None;
            }
        } else {
            self.p.vy += GRAV * self.gm * dt;
        }
        self.p.y += self.p.vy * dt;
        if rev_done {
            self.spawn_revive_plat();
        }
        if self.p.imm > 0.0 {
            self.p.imm -= dt;
        }
        if self.p.wing > 0.0 {
            self.p.wing -= dt;
        }
        if self.p.mag > 0.0 {
            self.p.mag = (self.p.mag - dt).max(0.0);
        }
        if !self.p.dead && self.p.rev <= 0.0 && self.p.y < self.h - 70.0 {
            self.safe_x = self.p.x;
            self.safe_y = self.p.y;
        }
        let trl = self.sv.eq_get("trl");
        if !trl.is_empty() && !self.p.dead && chance() < dt * 28.0 {
            let (x, y) = (self.p.x, self.p.y - 20.0);
            match trl.as_str() {
                "star" => self.fx(x, y, 1, &["#f3d23c", "#ffffff", "#f7b2ae"], 25.0, 0.0, 0.6, 0.55, K_STAR, 0.0),
                "conf" => self.fx(x, y, 1, &STARC, 25.0, 80.0, 0.7, 1.0, K_CONF, 0.0),
                _ => self.fx(x, y, 1, &["#ffffff", "#b9e3f5"], 20.0, -60.0, 0.8, 1.1, K_DOT, 0.0),
            }
        }
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
                self.parts.push(Part { x, y, vx: rnd(-20.0, 20.0), vy: rnd(150.0, 260.0), life: l, max: l, size: rnd(3.0, 6.0), col: c, kind: K_DOT, rot: 0.0, vr: 0.0, grav: 0.0 });
            }
        } else if self.p.pu == Item::Prop && chance() < 0.5 {
            let (x, y) = (self.p.x + rnd(-14.0, 14.0), self.p.y - 66.0);
            self.fx(x, y, 1, &["#f3d23c", "#ffffff"], 30.0, 120.0, 0.6, 0.7, K_STAR, 0.0);
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
            if p.pop > 0.0 {
                p.pop = (p.pop - dt * 2.6).max(0.0);
            }
        }
        for i in 0..self.plats.len() {
            if self.plats[i].tgt > 0.0 && !self.plats[i].broken {
                self.plats[i].tgt -= dt;
                if self.plats[i].tgt <= 0.0 {
                    self.strike(i);
                }
            }
            let p = &self.plats[i];
            if p.burnt && !p.broken && chance() < 0.04 {
                let (x, y) = (p.x + rnd(6.0, p.w - 6.0), p.y);
                self.add_part(x, y, rnd(-8.0, 8.0), -30.0, 1.0, rnd(2.0, 4.0), "#9b8c80", K_DOT, -10.0, 0.0, 0.0);
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
                            self.mis_add("pu", 1);
                            self.snd("power");
                            let (x, y) = (self.p.x, self.p.y - 30.0);
                            self.ring(x, y, "#6cc3ee");
                            self.fx(x, y, 10, &STARC, 160.0, 200.0, 0.8, 1.0, K_STAR, 0.0);
                            break;
                        }
                    }
                }
            }
            // aterrizaje
            if self.p.vy > 0.0 && self.p.pu == Item::None && self.p.rev <= 0.0 {
                for i in 0..self.plats.len() {
                    let p = self.plats[i].clone();
                    if p.broken || p.gone {
                        continue;
                    }
                    if prev_y <= p.y + 2.0 && self.p.y >= p.y && self.p.x + 13.0 > p.x && self.p.x - 13.0 < p.x + p.w {
                        self.combo_land(p.y - self.cam_off);
                        self.sv.stat_add("jumps", 1);
                        self.mis_add("jumps", 1);
                        if p.t == PT::K || p.burnt {
                            self.p.y = p.y;
                            self.p.sq = 0.6;
                            self.p.vy = JUMP * 0.5;
                            self.break_plat(i, false);
                            continue;
                        }
                        self.p.y = p.y;
                        self.p.sq = 1.0;
                        let ix = p.x + p.ix;
                        let (px, py) = (self.p.x, self.p.y);
                        if p.item == Item::Spring && px + 13.0 > ix && px - 13.0 < ix + 14.0 {
                            self.p.vy = SPRING;
                            self.snd("spring");
                            self.plats[i].used = 1.0;
                            self.shake = 3.0;
                            self.fx(px, py, 8, &STARC, 150.0, 300.0, 0.7, 1.0, K_STAR, 0.0);
                        } else if p.item == Item::Tramp && px + 13.0 > ix && px - 13.0 < ix + 38.0 {
                            self.p.vy = TRAMP;
                            self.snd("tramp");
                            self.plats[i].used = 1.0;
                            self.p.flip = 1.0;
                            self.shake = 4.0;
                            self.ring(px, py, "#e8504a");
                            self.fx(px, py, 14, &STARC, 220.0, 300.0, 0.9, 1.0, K_STAR, 0.0);
                        } else {
                            self.p.vy = JUMP;
                            self.snd("jump");
                            self.fx(px, py, 6, &["#ffffff", "#f3e6d4", if p.snow > 0.3 { "#ffffff" } else { pc(p.t).0 }], 70.0, -40.0, 0.6, 1.3, K_DOT, -10.0);
                        }
                        if p.t == PT::W {
                            self.plats[i].gone = true;
                            self.snd("puff");
                            self.fx(p.x + 29.0, p.y + 7.0, 12, &["#ffffff", "#eeeeee"], 90.0, -30.0, 0.7, 1.8, K_DOT, 0.0);
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
            if m.fvx != 0.0 {
                m.x += m.fvx * dt;
            }
            m.bx = (m.ph * 0.7).sin() * if m.fvx != 0.0 { 10.0 } else if m.bat { 40.0 } else { 14.0 };
            if m.dead {
                m.vy += GRAV * dt;
                m.y += m.vy * dt;
            }
        }
        // monedas
        let mut got = vec![];
        let mut mag_fx = vec![];
        let mut zip = false;
        let (pmag, pdead, ppx, ppy) = (self.p.mag, self.p.dead, self.p.x, self.p.y);
        for c in &mut self.coins {
            if pmag > 0.0 && !c.got && !pdead {
                let (mdx, mdy) = (ppx - c.x, (ppy - 24.0) - c.y);
                let md = mdx.hypot(mdy);
                if md < MAG_R {
                    if !c.mag {
                        zip = true;
                    }
                    c.mag = true;
                    let mf = ((380.0 + (MAG_R - md) * 2.5) * dt / md.max(1.0)).min(1.0);
                    c.x += mdx * mf;
                    c.y += mdy * mf;
                    mag_fx.push((c.x, c.y));
                }
            }
            if c.vy != 0.0 && !c.mag {
                c.y += c.vy * dt;
            }
            if !c.got && !self.p.dead && (self.p.x - c.x).abs() < 24.0 && c.y > self.p.y - 54.0 && c.y < self.p.y + 6.0 {
                c.got = true;
                got.push((c.x, c.y));
            }
        }
        self.mag_snd_t = (self.mag_snd_t - dt).max(0.0);
        if zip && self.mag_snd_t <= 0.0 {
            self.snd("zip");
            self.mag_snd_t = 0.08;
        }
        for (x, y) in mag_fx {
            self.add_part(x, y, rnd(-10.0, 10.0), rnd(-10.0, 10.0), 0.45, 3.0, "#f3d23c", K_DOT, 0.0, 0.0, 0.0);
        }
        for (x, y) in got {
            self.sv.coins += 1;
            self.run_coins += 1;
            self.sv.stat_add("coins", 1);
            self.mis_add("coins", 1);
            self.snd("coin");
            self.fx(x, y, 5, &["#f3d23c", "#ffffff"], 90.0, 100.0, 0.5, 0.6, K_STAR, 0.0);
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
        self.update_wx(dt);
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
            for q in &mut self.drops { q.y += d; }
            for q in &mut self.rocks { q.y += d; }
            for q in &mut self.bolts { q.y += d; }
            for q in &mut self.lost { q.1 += d; }
        }
        let h = self.h;
        self.plats.retain(|p| p.y < h + 40.0 && !p.gone);
        let w = self.w;
        self.mons.retain(|m| m.y < h + 80.0 && (m.fvx == 0.0 || (m.x > -170.0 && m.x < w + 170.0)));
        self.lost.retain(|l| l.1 < h + 40.0);
        self.coins.retain(|c| !c.got && c.y < h + 40.0);
        self.gen();
        if self.combo > 0 && self.p.pu == Item::None && self.p.rev <= 0.0 && (self.p.y - self.cam_off) > self.combo_y + 160.0 {
            self.combo_break();
        }
        self.combo_pop = (self.combo_pop - dt * 4.0).max(0.0);
        if !self.night_noted && self.night_k() > 0.35 && self.note.is_none() {
            self.night_noted = true;
            self.show_note("¡cae la noche!");
        }
        if self.p.imm > 0.0 && !self.p.dead && self.p.y > self.h - 20.0 {
            self.p.y = self.h - 20.0;
            self.p.vy = JUMP * 1.1;
            self.snd("jump");
        }
        self.ach_t -= dt;
        if self.ach_t <= 0.0 {
            self.ach_t = 0.5;
            self.track_alt();
            self.check_ach();
        }
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
        if let Some(t) = self.wx.ty {
            if matches!(self.state, State::Play | State::Pause | State::Over) {
                let c = wx_tint(t);
                g.rect(-20.0, -20.0, self.w + 40.0, self.h + 40.0, Color::new(c.0 / 255.0, c.1 / 255.0, c.2 / 255.0, self.wx.fade * if t == Wx::Storm { 0.18 } else { 0.09 }));
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
        let (mut c, mut d) = pc(p.t);
        if p.burnt {
            c = "#5a4840";
            d = "#3e302a";
        }
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
        if p.pop > 0.0 {
            let sc = (1.0 - p.pop) * (1.0 + (PI * (1.0 - p.pop)).sin() * 0.22);
            g.tr(29.0 * (1.0 - sc), 7.0 * (1.0 - sc));
            g.sc(sc, sc);
        }
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
        if p.burnt {
            g.stroke(&[vec2(14.0, 2.0), vec2(20.0, 8.0), vec2(26.0, 4.0)], 1.4, "#2b2420", false);
            g.stroke(&[vec2(36.0, 3.0), vec2(42.0, 9.0)], 1.4, "#2b2420", false);
        }
        if p.cor > 0.0 {
            let n = (p.cor * 7.0).ceil() as i32;
            for i in 0..n {
                g.fcirc(6.0 + (i as f32 * 23.0 + p.w * 3.0) % (p.w - 12.0), 4.0 + ((i * 5) % 7) as f32, 2.0 + p.cor * 2.2, "#4f7a1fcc", 0.0);
            }
        }
        if p.snow > 0.02 {
            let th = 2.0 + p.snow * 4.0;
            let mut cap = vec![vec2(2.0, 4.0)];
            for i in 0..=8 {
                let xx = 2.0 + i as f32 * 6.75;
                cap.push(vec2(xx, 4.0 - th - ((i as f32 * 1.7 + p.sw).sin() * 1.2 + 1.2) * p.snow));
            }
            cap.push(vec2(56.0, 4.0));
            g.fill(&cap, "#ffffff");
            g.stroke(&cap, 1.4, "#b9d6e6", false);
        }
        g.restore();
        if p.tgt > 0.0 {
            g.save();
            g.tr(p.x + p.w / 2.0, p.y + 7.0);
            g.rot(self.time * 3.0);
            g.dash_circle(0.0, 0.0, 24.0 + p.tgt * 10.0, 2.2, "#e8504a");
            g.dash_circle(0.0, 0.0, 10.0, 2.2, "#e8504a");
            g.restore();
        }
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

    fn cosm(&self, g: &mut Painter, l: &Look, ex: f32, pu: Item) {
        let a = l.acc.as_str();
        let h = if pu == Item::Prop { "" } else { l.hat.as_str() };
        match a {
            "shades" => {
                g.fr(-15.0 + ex, -38.0, 12.0, 8.0, 3.0, INK, 0.0);
                g.fr(3.0 + ex, -38.0, 12.0, 8.0, 3.0, INK, 0.0);
                g.ln(-3.0 + ex, -35.0, 3.0 + ex, -35.0, 2.0, INK);
            }
            "round" => {
                g.stroke(&ell_pts(-8.0 + ex, -33.0, 6.5, 6.5), 1.9, INK, true);
                g.stroke(&ell_pts(8.0 + ex, -33.0, 6.5, 6.5), 1.9, INK, true);
                g.ln(-1.5 + ex, -33.0, 1.5 + ex, -33.0, 1.9, INK);
            }
            "stache" => {
                g.fe(-4.0 + ex, -25.5, 5.0, 2.4, "#4a2822", 0.0);
                g.fe(4.0 + ex, -25.5, 5.0, 2.4, "#4a2822", 0.0);
            }
            "scarf" => {
                g.fr(-14.0, -21.0, 28.0, 7.0, 3.0, "#e8504a", 2.0);
                g.fr(5.0, -16.0, 7.0, 13.0, 2.0, "#e8504a", 2.0);
            }
            _ => {}
        }
        match h {
            "party" => {
                let tri = [vec2(-9.0, -50.0), vec2(9.0, -50.0), vec2(0.0, -77.0)];
                g.fill(&tri, "#e8504a");
                g.stroke(&tri, 2.0, INK, true);
                g.fcirc(0.0, -78.0, 3.2, "#f3d23c", 1.5);
            }
            "crown" => {
                let cp = [vec2(-11.0, -50.0), vec2(-11.0, -64.0), vec2(-5.0, -57.0), vec2(0.0, -67.0), vec2(5.0, -57.0), vec2(11.0, -64.0), vec2(11.0, -50.0)];
                g.fill(&cp, "#f3d23c");
                g.stroke(&cp, 2.0, INK, true);
            }
            "top" => {
                g.fr(-10.0, -76.0, 20.0, 26.0, 3.0, INK, 0.0);
                g.rect(-10.0, -59.0, 20.0, 5.0, hx("#e8504a"));
                g.fr(-18.0, -53.0, 36.0, 6.0, 3.0, INK, 1.5);
            }
            "halo" => {
                g.stroke(&ell_pts(0.0, -66.0 + (self.time * 3.0).sin() * 1.5, 13.0, 4.0), 4.0, "#f3d23c", true);
            }
            "cap" => {
                let dome = arc_pts(0.0, -48.0, 16.0, PI, 0.0, 10);
                g.fill(&dome, "#4a9be0");
                g.stroke(&dome, 2.0, INK, true);
                g.fr(2.0, -51.0, 22.0, 5.0, 2.5, "#4a9be0", 2.0);
            }
            "flower" => {
                for i in 0..5 {
                    let t = i as f32 * 1.2566;
                    g.fcirc(13.0 + t.cos() * 4.5, -58.0 + t.sin() * 4.5, 3.6, "#f7b2ae", 1.3);
                }
                g.fcirc(13.0, -58.0, 3.0, "#f3d23c", 1.3);
            }
            _ => {}
        }
    }
    fn wings(&self, g: &mut Painter, k: f32) {
        let flap = (self.time * 14.0).sin() * 0.2;
        for s in [-1.0f32, 1.0] {
            g.save();
            g.tr(s * 8.0, -26.0);
            g.sc(s * k, k);
            for i in 0..3 {
                g.save();
                g.rot(0.15 - i as f32 * 0.45 + flap);
                let l = 40.0 - i as f32 * 6.0;
                g.fe(l / 2.0, 0.0, l / 2.0, 7.5 - i as f32, "#ffffff", 2.0);
                g.restore();
            }
            g.restore();
        }
    }
    fn bubble(&self, g: &mut Painter) {
        let pul = 1.0 + (self.time * 3.0).sin() * 0.02;
        let pts = ell_pts(0.0, -30.0, 34.0 * pul, 42.0 * pul);
        g.fill(&pts, "#9fd6f244");
        g.stroke(&pts, 2.0, "#6cb6e0", true);
        g.stroke(&arc_pts(0.0, -30.0, 27.0, PI * 1.12, PI * 1.45, 6), 2.4, "#ffffff", false);
    }
    fn magnet_icon(&self, g: &mut Painter, x: f32, y: f32, s: f32) {
        g.save();
        g.tr(x, y);
        g.sc(s, s);
        let u = arc_pts(0.0, 0.0, 8.0, 0.0, PI, 8);
        for (lw, c) in [(9.0, INK), (5.5, "#e8504a")] {
            g.ln(8.0, 0.0, 8.0, -10.0, lw, c);
            g.ln(-8.0, 0.0, -8.0, -10.0, lw, c);
            g.stroke(&u, lw, c, false);
        }
        g.ln(8.0, -10.0, 8.0, -6.5, 5.5, "#ffffff");
        g.ln(-8.0, -10.0, -8.0, -6.5, 5.5, "#ffffff");
        g.restore();
    }
    fn pow_icon(&self, g: &mut Painter, id: &str, x: f32, y: f32) {
        g.save();
        g.tr(x, y);
        match id {
            "magnet" => self.magnet_icon(g, 0.0, 8.0, 2.1),
            "shield" => {
                g.save();
                g.tr(0.0, 28.0);
                g.sc(0.85, 0.85);
                self.bubble(g);
                g.restore();
                g.fill(&star_pts(7.0), "#f3d23c");
                g.stroke(&star_pts(7.0), 1.5, INK, true);
            }
            _ => {
                g.fr(-26.0, 6.0, 52.0, 13.0, 6.0, "#86d443", 2.0);
                let ar = arc_pts(0.0, -9.0, 11.0, PI * 0.15, PI * 1.75, 12);
                g.stroke(&ar, 3.0, INK, false);
                let e = *ar.last().unwrap();
                g.fill(&[e + vec2(-5.0, -6.0), e + vec2(5.0, -5.0), e + vec2(0.0, 5.0)], "#f3d23c");
                g.fcirc(15.0, -17.0, 2.5, "#f3d23c", 1.2);
            }
        }
        g.restore();
    }

    #[allow(clippy::too_many_arguments)]
    fn bubu(&self, g: &mut Painter, x: f32, y: f32, face: f32, sq: f32, shooting: bool, flip: f32, pu: Item, l: &Look, wk: f32, shield: bool, mag: bool) {
        g.save();
        g.tr(x, y);
        if flip > 0.0 {
            g.tr(0.0, -28.0);
            g.rot((1.0 - flip) * TAU * face);
            g.tr(0.0, 28.0);
        }
        g.sc(1.0 + sq * 0.12, 1.0 - sq * 0.12);
        let dd = l.dudu;
        let mut b = if dd { "#d49b86" } else { "#ffffff" };
        if let Some(c) = SHOP_COL.iter().find(|c| c.0 == l.col) {
            b = c.3;
        }
        if pu == Item::Jet {
            g.save();
            g.tr(-face * 16.0 - 9.0, -38.0);
            self.jet(g, 0.0, 0.0, true);
            g.restore();
        }
        let ex = face * 1.5;
        if wk > 0.0 {
            self.wings(g, wk);
        }
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
        if !dd && l.acc != "scarf" {
            g.fill(&[vec2(0.0, -14.0), vec2(-7.0, -18.0), vec2(-7.0, -10.0)], "#4a2822");
            g.fill(&[vec2(0.0, -14.0), vec2(7.0, -18.0), vec2(7.0, -10.0)], "#4a2822");
            g.fcirc(0.0, -14.0, 2.4, "#4a2822", 0.0);
        }
        let up = if shooting { -4.0 } else { 0.0 };
        for s in [-1.0f32, 1.0] {
            g.fe(s * 13.0, -17.0 + up, 4.5, 5.5, b, 2.0);
        }
        self.cosm(g, l, ex, pu);
        if l.acc != "shades" {
            if self.time % 3.4 < 0.12 {
                g.ln(-11.0 + ex, -33.0, -5.0 + ex, -33.0, 2.0, INK);
                g.ln(5.0 + ex, -33.0, 11.0 + ex, -33.0, 2.0, INK);
            } else {
                g.fcirc(-8.0 + ex, -33.0, 2.8, INK, 0.0);
                g.fcirc(8.0 + ex, -33.0, 2.8, INK, 0.0);
                g.fcirc(-7.2 + ex, -34.0, 0.9, "#ffffff", 0.0);
                g.fcirc(8.8 + ex, -34.0, 0.9, "#ffffff", 0.0);
            }
        }
        if shooting {
            g.fe(ex, -27.0, 2.6, 3.0, INK, 0.0);
        } else {
            let mut sm = arc_pts(-1.9 + ex, -29.0, 1.9, 0.0, PI, 6);
            sm.extend(arc_pts(1.9 + ex, -29.0, 1.9, 0.0, PI, 6));
            g.stroke(&sm, 1.6, INK, false);
        }
        if pu == Item::Prop {
            self.hat(g, 0.0, -51.0, self.time);
        }
        if shield {
            self.bubble(g);
        }
        if mag {
            self.magnet_icon(g, 0.0, -86.0 + (self.time * 4.0).sin() * 2.0, 1.0);
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

    fn paint_rocks(&self, g: &mut Painter) {
        for k in &self.rocks {
            g.save();
            g.tr(k.x, k.y);
            g.rot(k.rot);
            let n = k.pts.len();
            let pts: Vec<Vec2> = (0..n).map(|i| {
                let a = i as f32 / n as f32 * TAU;
                vec2(a.cos() * k.pts[i], a.sin() * k.pts[i])
            }).collect();
            g.fill(&pts, if k.hail { "#e6f3fb" } else { "#8a7466" });
            g.stroke(&pts, 2.2, INK, true);
            let cc = if k.hail { "#b8d4e6" } else { "#5e4c42" };
            g.fcirc(-k.r * 0.3, -k.r * 0.2, k.r * 0.22, cc, 0.0);
            g.fcirc(k.r * 0.35, k.r * 0.25, k.r * 0.15, cc, 0.0);
            g.restore();
        }
        for (x, t) in &self.warns {
            if ((t * 10.0).floor() as i32) % 2 == 1 {
                continue;
            }
            g.save();
            g.tr(*x, 62.0 + (self.time * 20.0).sin() * 1.5);
            let tri = [vec2(0.0, -16.0), vec2(15.0, 12.0), vec2(-15.0, 12.0)];
            g.fill(&tri, "#f3d23c");
            g.stroke(&tri, 2.0, INK, true);
            g.txt("!", 0.0, 2.0, 22.0, INK, 1, 0.0);
            g.restore();
        }
    }
    fn paint_drops(&self, g: &mut Painter) {
        let wind = self.wx.wind;
        let mut rain = hx("#3d6fc4");
        rain.a *= 0.75 * g.ga;
        let mut acid = hx("#5fbf2a");
        acid.a *= 0.75 * g.ga;
        for d in &self.drops {
            if d.snow {
                g.fcirc(d.x, d.y, 2.6, "#ffffff", 0.0);
                continue;
            }
            let dx = (d.vx + wind * 140.0) * 0.025;
            let pts = [vec2(d.x, d.y), vec2(d.x - dx, d.y - d.vy * 0.022)];
            g.stroke_c(&pts, if d.acid { 2.0 } else { 1.6 }, if d.acid { acid } else { rain }, false);
            if d.acid {
                g.fill_c(&ell_pts(d.x, d.y, 1.8, 1.8), acid);
            }
        }
    }
    fn paint_bolts(&self, g: &mut Painter) {
        for b in &self.bolts {
            let a = 1.0 - b.life / b.mx;
            g.save();
            g.ga *= a;
            let mut pts = vec![vec2(b.x + rnd(-30.0, 30.0), -10.0)];
            let n = 7;
            for i in 1..n {
                pts.push(vec2(b.x + rnd(-22.0, 22.0), -10.0 + (b.y + 10.0) * i as f32 / n as f32));
            }
            pts.push(vec2(b.x, b.y));
            for (lw, c) in [(9.0, "#fff7c0"), (4.5, "#f3d23c"), (1.6, INK)] {
                g.stroke(&pts, lw, c, false);
            }
            g.restore();
        }
    }
    fn fog(&self, g: &mut Painter) {
        if self.wx.ty != Some(Wx::Fog) || self.wx.fade <= 0.0 {
            return;
        }
        let a = 0.9 * self.wx.fade * g.ga;
        let c0 = Color::new(240.0 / 255.0, 236.0 / 255.0, 230.0 / 255.0, 0.0);
        let c1 = Color::new(c0.r, c0.g, c0.b, a);
        let ctr = vec2(self.p.x, self.p.y - 30.0);
        let m = 28;
        let mut verts = vec![];
        let mut idx: Vec<u16> = vec![];
        for i in 0..m {
            let an = TAU * i as f32 / m as f32;
            let d = vec2(an.cos(), an.sin());
            for (r, c) in [(110.0, c0), (300.0, c1), (900.0, c1)] {
                let p = g.tp(ctr + d * r);
                verts.push(macroquad::models::Vertex::new(p.x, p.y, 0.0, 0.0, 0.0, c));
            }
        }
        for i in 0..m {
            let j = (i + 1) % m;
            let (a0, b0) = ((i * 3) as u16, (j * 3) as u16);
            idx.extend_from_slice(&[a0, a0 + 1, b0 + 1, a0, b0 + 1, b0, a0 + 1, a0 + 2, b0 + 2, a0 + 1, b0 + 2, b0 + 1]);
        }
        draw_mesh(&Mesh { vertices: verts, indices: idx, texture: None });
        let cp = cloud_pts();
        for i in 0..3 {
            g.save();
            g.ga *= self.wx.fade * 0.8;
            g.tr(fposmod(self.time * 14.0 + i as f32 * 140.0, self.w + 200.0) - 100.0, 150.0 + i as f32 * 170.0);
            g.sc(2.6, 2.6);
            g.fill(&cp, "#f6f2ec");
            g.restore();
        }
    }
    fn paint_note(&self, g: &mut Painter) {
        let Some((txt, t)) = &self.note else { return };
        let t = *t;
        let k = (t / 0.35).min(1.0).min(((3.4 - t) / 0.4).clamp(0.0, 1.0));
        let ek = 1.0 - (1.0 - k).powi(3);
        g.save();
        g.tr(self.w / 2.0, 96.0 - (1.0 - ek) * 70.0);
        g.rot(-0.05 + (self.time * 2.0).sin() * 0.015);
        g.ga *= k;
        let tw = 150f32.max(txt.chars().count() as f32 * 11.0 + 40.0);
        let th = 46.0;
        g.rect(-tw / 2.0 + 4.0, -th / 2.0 + 5.0, tw, th, hx("#3b242022"));
        let pp = [vec2(-tw / 2.0, -th / 2.0), vec2(tw / 2.0, -th / 2.0), vec2(tw / 2.0, th / 2.0 - 6.0), vec2(tw / 2.0 - 8.0, th / 2.0), vec2(-tw / 2.0, th / 2.0)];
        g.fill(&pp, "#fff8d6");
        g.stroke(&pp, 2.0, INK, true);
        g.ln(-tw / 2.0 + 8.0, th / 2.0 - 9.0, tw / 2.0 - 12.0, th / 2.0 - 9.0, 1.0, "#e8b8a8");
        g.save();
        g.tr(0.0, -th / 2.0);
        g.rot(0.06);
        g.rect(-22.0, -8.0, 44.0, 15.0, hx("#cfe6f0b0"));
        g.restore();
        g.txt(txt, 0.0, 2.0, 24.0, INK, 1, 0.0);
        g.restore();
    }
    fn paint_parts(&self, g: &mut Painter) {
        for p in &self.parts {
            let k = (p.life / p.max).clamp(0.0, 1.0);
            g.save();
            g.ga *= k.min(1.0);
            let mut c = p.col;
            c.a *= g.ga;
            let (s, co) = p.rot.sin_cos();
            let rot = |v: &Vec2| vec2(p.x + v.x * co - v.y * s, p.y + v.x * s + v.y * co);
            match p.kind {
                K_STAR => {
                    let pts: Vec<Vec2> = star_pts(p.size).iter().map(rot).collect();
                    g.fill_c(&pts, c);
                }
                K_LEAF => {
                    let pts: Vec<Vec2> = ell_pts(0.0, 0.0, p.size * 1.6, p.size * 0.8).iter().map(rot).collect();
                    g.fill_c(&pts, c);
                }
                K_CONF => {
                    let q = [vec2(-3.0, -1.5), vec2(3.0, -1.5), vec2(3.0, 1.5), vec2(-3.0, 1.5)];
                    let pts: Vec<Vec2> = q.iter().map(rot).collect();
                    g.fill_c(&pts, c);
                }
                K_GUST => {
                    let mut c2 = c;
                    c2.a *= 0.5;
                    g.stroke_c(&[vec2(p.x, p.y), vec2(p.x - p.vx * 0.06, p.y)], p.size * 0.6, c2, false);
                }
                _ => g.fill_c(&ell_pts(p.x, p.y, p.size * (0.4 + k * 0.6), p.size * (0.4 + k * 0.6)), c),
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
        self.paint_rocks(g);
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
        if self.p.mag > 0.0 {
            g.save();
            g.ga *= 0.3;
            g.tr(self.p.x, self.p.y - 24.0);
            g.rot(self.time);
            g.dash_circle(0.0, 0.0, MAG_R * (0.97 + (self.time * 5.0).sin() * 0.03), 2.0, "#e8504a");
            g.restore();
        }
        g.save();
        if self.p.imm > 0.0 && self.p.rev <= 0.0 {
            g.ga *= 0.5 + 0.5 * (self.time * 12.0).sin().abs();
        }
        let wk = (self.p.wing / 0.5).clamp(0.0, 1.0);
        self.bubu(g, self.p.x, self.p.y, self.p.face, self.p.sq, self.p.shoot_t > 0.0, self.p.flip, self.p.pu, &self.sv.look(), wk, self.p.shield, self.p.mag > 0.0);
        g.restore();
        self.paint_drops(g);
        self.paint_bolts(g);
        self.fog(g);
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
        if self.wx.ty.is_some() {
            let k = (1.0 - self.wx.t / self.wx.dur).max(0.0);
            g.save();
            g.ga *= 0.8;
            g.fr(110.0, 30.0, 110.0 * k, 4.0, 2.0, "#7a6355", 0.0);
            g.restore();
        }
        if self.p.mag > 0.0 {
            g.fcirc(w - 34.0, 66.0, 19.0, "#ffffff", 2.0);
            self.magnet_icon(g, w - 34.0, 72.0, 0.9);
            let a0 = -PI / 2.0;
            let arc = arc_pts(w - 34.0, 66.0, 23.0, a0, a0 + TAU * (self.p.mag / MAG_T).max(0.01), 28);
            g.stroke(&arc, 3.0, "#e8504a", false);
        } else if self.sv.has_pw("magnet") {
            g.save();
            g.ga *= 0.8 + (self.time * 5.0).sin() * 0.2;
            g.fcirc(w - 34.0, 66.0, 19.0, "#f3d23c", 2.0);
            self.magnet_icon(g, w - 34.0, 72.0, 0.9);
            g.restore();
            self.btns.push(Btn { x: w - 56.0, y: 44.0, w: 44.0, h: 44.0, id: 12 });
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
        g.fr(ox + 40.0, y, 280.0, 34.0, 12.0, "#ffffff", 2.0);
        g.txt(label, ox + BASE_W / 2.0, y + 18.0, 22.0, INK, 1, 0.0);
        for (s, x) in [(-1.0f32, ox + 58.0), (1.0, ox + 302.0)] {
            g.stroke(&[vec2(x - s * 3.5, y + 9.0), vec2(x + s * 3.5, y + 17.0), vec2(x - s * 3.5, y + 25.0)], 2.6, INK, false);
        }
        self.btns.push(Btn { x: ox + 40.0, y, w: 90.0, h: 34.0, id: id_l });
        self.btns.push(Btn { x: ox + 130.0, y, w: 190.0, h: 34.0, id: id_r });
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
            self.fx(x, y, 1, &STARC, 20.0, -20.0, 1.2, 0.8, K_STAR, 0.0);
        }
        self.paint_parts(g);
        let by = 430.0 - (self.time * 3.2).sin().abs() * 120.0;
        let p0 = self.mk(cx - 29.0, 432.0, PT::G);
        self.plat(g, &p0);
        self.bubu(g, cx, by, 1.0, if by > 420.0 { 1.0 } else { 0.0 }, false, self.swap_t, Item::None, &self.sv.look(), 0.0, false, false);
        let bat = Mon { x: ox + 80.0, y: 300.0 + (self.time * 2.0).sin() * 10.0, bx: 0.0, bat: true, w: 56.0, h: 34.0, ph: self.time * 2.0, dead: false, vy: 0.0, fvx: 0.0 };
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
        self.sel_row(g, ox, 452.0, &format!("modo: {}", mode.1), 1, 2);
        self.sel_row(g, ox, 490.0, &format!("nivel: {}", DIFS[self.sv.dif].0), 3, 4);
        self.button(g, "jugar", cx - 100.0, 540.0, 200.0, 0, "#86d443");
        self.button(g, "tienda", ox + 40.0, 590.0, 135.0, 7, "#f3d23c");
        self.button(g, "opciones", ox + 185.0, 590.0, 135.0, 53, "#ffffff");
        // botones de arriba: diario, logros, música
        let pend = self.daily_pending();
        g.fr(8.0, 8.0, 44.0, 40.0, 12.0, if pend { "#fff6c8" } else { "#ffffff" }, 2.0);
        g.save();
        g.tr(30.0, 32.0);
        if pend {
            g.rot((self.time * 9.0).sin() * 0.08 * (self.time * 1.6).sin().max(0.0));
        }
        self.chest_draw(g, 0.42, 0.0, false);
        g.restore();
        if pend {
            g.fcirc(48.0, 11.0, 6.0, "#e8504a", 1.6);
        }
        self.btns.push(Btn { x: 8.0, y: 8.0, w: 44.0, h: 40.0, id: 50 });
        g.fr(58.0, 8.0, 44.0, 40.0, 12.0, "#ffffff", 2.0);
        self.medal(g, 80.0, 26.0, 0.62, true);
        self.btns.push(Btn { x: 58.0, y: 8.0, w: 44.0, h: 40.0, id: 51 });
        let w = self.w;
        g.fr(w - 52.0, 8.0, 44.0, 40.0, 12.0, "#ffffff", 2.0);
        self.note_icon(g, w - 31.0, 28.0, self.sv.music);
        self.btns.push(Btn { x: w - 52.0, y: 8.0, w: 44.0, h: 40.0, id: 52 });
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
        g.save();
        g.tr(cx, 330.0 + (self.time * 4.0).sin() * 3.0);
        g.sc(0.85, 0.85);
        self.bubu(g, 0.0, 0.0, 1.0, 0.0, false, 0.0, Item::None, &self.sv.look(), 0.0, false, false);
        g.restore();
        if !(self.timed() && self.tl <= 0.0) {
            let cost = self.revive_cost();
            let col = if self.sv.coins >= cost { "#f3d23c" } else { "#dddddd" };
            self.button(g, &format!("revivir · {} monedas", cost), cx - 130.0, 340.0, 260.0, 13, col);
        }
        self.button(g, "otra vez", cx - 85.0, 400.0, 170.0, 0, "#86d443");
        self.button(g, "menú", cx - 85.0, 452.0, 170.0, 6, "#ffffff");
        g.restore();
    }

    fn shop_screen(&mut self, g: &mut Painter) {
        let ox = (self.w - BASE_W) / 2.0;
        let cx = self.w / 2.0;
        self.title(g, "tienda", ox + 84.0, 46.0, 46.0, -0.04);
        g.fcirc(ox + BASE_W - 78.0, 36.0, 9.0, "#f3d23c", 2.0);
        g.txt(&self.sv.coins.to_string(), ox + BASE_W - 64.0, 38.0, 26.0, INK, 0, 0.0);
        let cat = TABS[self.shop_tab].0;
        let items = shop_items(cat);
        if !items.iter().any(|i| i.0 == self.shop_sel) {
            self.shop_sel = self.sv.eq_get(cat);
            if self.shop_sel.is_empty() {
                self.shop_sel = items[0].0.to_string();
            }
        }
        let desc = if cat == "pow" { pow_desc(&self.shop_sel) } else { "toca al osito: bubu / dudu" };
        g.txt(desc, cx, 96.0, 15.0, "#7a6355", 1, 0.0);
        let p0 = self.mk(cx - 29.0, 228.0, PT::G);
        self.plat(g, &p0);
        let mut lk = self.sv.look();
        let (mut sh, mut mg) = (false, false);
        if cat == "pow" {
            sh = self.shop_sel == "shield";
            mg = self.shop_sel == "magnet";
        } else {
            match cat {
                "hat" => lk.hat = self.shop_sel.clone(),
                "acc" => lk.acc = self.shop_sel.clone(),
                "col" => lk.col = self.shop_sel.clone(),
                _ => {}
            }
        }
        g.save();
        g.tr(cx, 228.0);
        g.sc(1.5, 1.5);
        self.bubu(g, 0.0, 0.0, 1.0, 0.0, false, self.swap_t, Item::None, &lk, 0.0, sh, mg);
        g.restore();
        self.btns.push(Btn { x: cx - 42.0, y: 132.0, w: 84.0, h: 108.0, id: 5 });
        for (i, t) in TABS.iter().enumerate() {
            let x = ox + 6.0 + i as f32 * 69.0;
            g.fr(x, 250.0, 65.0, 28.0, 10.0, if i == self.shop_tab { "#86d443" } else { "#ffffff" }, 2.0);
            g.txt(t.1, x + 32.5, 265.0, 16.0, INK, 1, 0.0);
            self.btns.push(Btn { x, y: 250.0, w: 65.0, h: 28.0, id: 20 + i as u8 });
        }
        for (i, it) in items.iter().enumerate() {
            let x = ox + 12.0 + (i % 3) as f32 * 112.0;
            let y = 288.0 + (i / 3) as f32 * 104.0;
            let on = it.0 == self.shop_sel;
            let has = if cat == "pow" { self.sv.has_pw(it.0) } else { self.sv.own.contains(&format!("{}:{}", cat, it.0)) };
            let eq = self.sv.eq_get(cat) == it.0;
            g.fr(x, y, 104.0, 98.0, 12.0, if on { "#fff6c8" } else { "#ffffff" }, if on { 3.0 } else { 2.0 });
            if cat == "pow" {
                self.pow_icon(g, it.0, x + 52.0, y + 36.0);
            } else {
                if cat == "trl" {
                    for j in 0..4 {
                        let tc = match it.0 { "star" => "#f3d23c", "conf" => STARC[j], _ => "#b9e3f5" };
                        g.fcirc(x + 18.0 + j as f32 * 5.0, y + 22.0 + j as f32 * 9.0, 3.2 - j as f32 * 0.5, tc, 1.0);
                    }
                }
                let mut l2 = self.sv.look();
                match cat {
                    "hat" => l2.hat = it.0.to_string(),
                    "acc" => l2.acc = it.0.to_string(),
                    "col" => l2.col = it.0.to_string(),
                    _ => {}
                }
                g.save();
                g.tr(x + 56.0, y + 58.0);
                g.sc(0.62, 0.62);
                self.bubu(g, 0.0, 0.0, 1.0, 0.0, false, 0.0, Item::None, &l2, 0.0, false, false);
                g.restore();
            }
            g.txt(it.1, x + 52.0, y + 76.0, 15.0, INK, 1, 0.0);
            if has {
                g.txt(if cat == "pow" || !eq { "tuyo" } else { "equipado" }, x + 52.0, y + 91.0, 15.0, "#3c8a1a", 1, 0.0);
            } else {
                g.fcirc(x + 40.0, y + 90.0, 4.5, "#f3d23c", 1.2);
                g.txt(&it.2.to_string(), x + 48.0, y + 91.0, 15.0, INK, 0, 0.0);
            }
            self.btns.push(Btn { x, y, w: 104.0, h: 98.0, id: 30 + i as u8 });
        }
        if let Some(it) = items.iter().find(|i| i.0 == self.shop_sel) {
            let has = if cat == "pow" { self.sv.has_pw(it.0) } else { self.sv.own.contains(&format!("{}:{}", cat, it.0)) };
            let (mut lbl, mut bc) = (format!("comprar · {}", it.2), if self.sv.coins >= it.2 { "#f3d23c" } else { "#dddddd" });
            if has {
                lbl = if self.sv.eq_get(cat) == it.0 { "quitar".into() } else { "equipar".into() };
                bc = "#86d443";
                if cat == "pow" {
                    lbl = "ya lo tienes".into();
                    bc = "#dddddd";
                }
            }
            self.button(g, &lbl, cx - 95.0, 498.0, 190.0, 40, bc);
        }
        self.button(g, "volver", cx - 70.0, 548.0, 140.0, 6, "#ffffff");
        self.paint_parts(g);
    }


    fn chest_draw(&self, g: &mut Painter, s: f32, open_k: f32, glow: bool) {
        g.save();
        g.sc(s, s);
        if glow {
            g.save();
            g.ga *= 0.55 + (self.time * 4.0).sin() * 0.15;
            for i in 0..8 {
                let a = i as f32 * TAU / 8.0 + self.time * 0.6;
                let tri = [vec2(0.0, -24.0), vec2((a - 0.12).cos() * 66.0, -24.0 + (a - 0.12).sin() * 66.0), vec2((a + 0.12).cos() * 66.0, -24.0 + (a + 0.12).sin() * 66.0)];
                g.fill(&tri, "#fff3a0");
            }
            g.fcirc(0.0, -24.0, 34.0, "#fff3a0", 0.0);
            g.restore();
        }
        g.fr(-32.0, -16.0, 64.0, 38.0, 6.0, "#b8865a", 0.0);
        g.rect(-31.0, -2.0, 62.0, 6.0, hx("#8a5e3a"));
        for x in [-24.0, 18.0] {
            g.rect(x, -15.0, 6.0, 36.0, hx("#d9b48a"));
        }
        g.stroke(&rr_pts(-32.0, -16.0, 64.0, 38.0, 6.0), 2.4, INK, true);
        g.save();
        g.tr(-32.0, -16.0 - open_k * 9.0);
        g.rot(-open_k * 0.45);
        g.fr(0.0, -20.0, 64.0, 20.0, 8.0, "#c9965f", 2.4);
        g.rect(8.0, -19.0, 6.0, 18.0, hx("#d9b48a"));
        g.rect(50.0, -19.0, 6.0, 18.0, hx("#d9b48a"));
        if open_k < 0.3 {
            g.fr(26.0, -8.0, 12.0, 14.0, 3.0, "#f3d23c", 2.0);
            g.fcirc(32.0, -1.0, 2.0, INK, 0.0);
        }
        g.restore();
        if open_k > 0.3 {
            g.fe(0.0, -16.0, 26.0, 5.0, "#f3d23c", 2.0);
            for i in 0..4 {
                g.fcirc(-15.0 + i as f32 * 10.0, -20.0 - (i % 2) as f32 * 3.0, 5.0, "#f3d23c", 1.5);
            }
        }
        g.restore();
    }
    fn medal(&self, g: &mut Painter, x: f32, y: f32, s: f32, on: bool) {
        g.save();
        g.tr(x, y);
        g.sc(s, s);
        let (c1, c2) = if on { ("#e8504a", "#6cc3ee") } else { ("#cfc4b6", "#ddd4c8") };
        let ra = [vec2(-12.0, -20.0), vec2(-2.0, -20.0), vec2(-6.0, 2.0), vec2(-14.0, 0.0)];
        let rb = [vec2(2.0, -20.0), vec2(12.0, -20.0), vec2(14.0, 0.0), vec2(6.0, 2.0)];
        g.fill(&ra, c1);
        g.stroke(&ra, 1.8, INK, true);
        g.fill(&rb, c2);
        g.stroke(&rb, 1.8, INK, true);
        g.fcirc(0.0, 10.0, 15.0, if on { "#f3d23c" } else { "#e8e0d4" }, 2.2);
        g.fcirc(0.0, 10.0, 10.5, if on { "#f7d95c" } else { "#efe8de" }, 1.2);
        let st: Vec<Vec2> = star_pts(7.0).iter().map(|v| *v + vec2(0.0, 10.0)).collect();
        g.fill(&st, if on { "#ffffff" } else { "#d8cfc2" });
        g.stroke(&st, 1.2, INK, true);
        g.restore();
    }
    fn note_icon(&self, g: &mut Painter, x: f32, y: f32, on: bool) {
        g.save();
        g.tr(x, y);
        g.fcirc(-5.0, 6.0, 4.0, INK, 0.0);
        g.fcirc(7.0, 3.0, 4.0, INK, 0.0);
        g.rect(-2.2, -10.0, 2.4, 16.0, hx(INK));
        g.rect(9.6, -13.0, 2.4, 16.0, hx(INK));
        g.ln(-1.0, -9.0, 11.0, -12.0, 4.0, INK);
        if !on {
            g.ln(-12.0, -14.0, 16.0, 14.0, 6.0, "#ffffff");
            g.ln(-12.0, -14.0, 16.0, 14.0, 3.2, "#e8504a");
        }
        g.restore();
    }
    fn paint_banner(&self, g: &mut Painter) {
        let Some((txt, kind, t)) = self.banners.first() else { return };
        if *t < 0.0 {
            return;
        }
        let k = (t / 0.25).min((2.8 - t) / 0.3).clamp(0.0, 1.0);
        let e = 1.0 - (1.0 - k).powi(3);
        let y = match self.state {
            State::Play => 132.0,
            State::Over | State::Pause => self.h - 44.0,
            _ => 26.0,
        };
        let w = txt.chars().count() as f32 * 9.0 + 70.0;
        g.save();
        g.tr(self.w / 2.0, y - (1.0 - e) * 50.0);
        g.rot(-0.02);
        g.ga *= k;
        g.fr(-w / 2.0 + 3.0, -17.0, w, 38.0, 14.0, "#3b242022", 0.0);
        g.fr(-w / 2.0, -21.0, w, 38.0, 14.0, "#fff8d6", 2.2);
        match kind {
            1 => self.medal(g, -w / 2.0 + 24.0, -9.0, 0.55, true),
            2 => {
                g.save();
                g.tr(-w / 2.0 + 24.0, -2.0);
                self.chest_draw(g, 0.36, 0.0, false);
                g.restore();
            }
            _ => {
                g.star(-w / 2.0 + 24.0, -2.0, 10.0, self.time * 2.0, "#f3d23c");
            }
        }
        g.txt(txt, 14.0, -2.0, 20.0, INK, 1, 0.0);
        g.restore();
    }
    fn coins_corner(&self, g: &mut Painter, ox: f32) {
        g.fcirc(ox + BASE_W - 78.0, 36.0, 9.0, "#f3d23c", 2.0);
        g.txt(&self.sv.coins.to_string(), ox + BASE_W - 64.0, 38.0, 26.0, INK, 0, 0.0);
    }
    fn daily_screen(&mut self, g: &mut Painter) {
        let ox = (self.w - BASE_W) / 2.0;
        let cx = self.w / 2.0;
        self.title(g, "diario", ox + 70.0, 46.0, 46.0, -0.04);
        self.coins_corner(g, ox);
        let st = self.sv.streak;
        g.txt(&format!("racha: {} {}", st, if st == 1 { "día seguido" } else { "días seguidos" }), cx, 90.0, 20.0, INK, 1, 0.0);
        let cur = st.clamp(1, 7) as usize;
        let ready = self.chest_ready();
        for i in 0..7 {
            let x = ox + 12.0 + i as f32 * 48.0;
            let done = i + 1 < cur || (i + 1 == cur && !ready);
            let now = i + 1 == cur;
            g.fr(x, 106.0, 44.0, 52.0, 10.0, if done { "#a8e6b8" } else if now { "#fff6c8" } else { "#ffffff" }, if now { 3.0 } else { 2.0 });
            g.txt(&format!("día {}{}", i + 1, if i == 6 { "+" } else { "" }), x + 22.0, 119.0, 14.0, INK, 1, 0.0);
            g.fcirc(x + 11.0, 143.0, 4.5, "#f3d23c", 1.2);
            g.txt(&format!("+{}", STREAK_R[i]), x + 18.0, 144.0, 15.0, INK, 0, 0.0);
        }
        let (mut shk, mut ok) = (0.0, 0.0);
        if self.chest_t >= 0.0 {
            if self.chest_t < 0.8 {
                shk = (self.chest_t * 50.0).sin() * 0.12 * (self.chest_t / 0.8);
            }
            ok = ((self.chest_t - 0.8) / 0.25).clamp(0.0, 1.0);
        } else if ready {
            shk = (self.time * 9.0).sin() * 0.05 * (self.time * 1.6).sin().max(0.0);
        }
        g.save();
        g.tr(cx, 264.0 - (self.time * 3.0).sin().abs() * if ready && self.chest_t < 0.0 { 4.0 } else { 0.0 });
        g.rot(shk);
        if !ready && self.chest_t < 0.0 {
            g.ga *= 0.55;
        }
        let ek = 1.0 - (1.0 - ok).powi(3);
        self.chest_draw(g, 1.55, ek, ok > 0.0 || (ready && self.chest_t < 0.0));
        g.restore();
        if self.chest_t >= 0.9 {
            let pk = ((self.chest_t - 0.9) * 4.0).min(1.0);
            g.save();
            g.tr(cx, 310.0);
            g.sc(0.6 + pk * 0.4, 0.6 + pk * 0.4);
            g.txt(&self.chest_prize.0, 0.0, 0.0, 30.0, "#e8504a", 1, 6.0);
            g.restore();
            let sub = &self.chest_prize.1;
            let line = format!("{}racha +{} monedas", if sub.is_empty() { String::new() } else { format!("{} · ", sub) }, self.chest_prize.2);
            g.txt(&line, cx, 337.0, 17.0, "#7a6355", 1, 0.0);
        } else if self.chest_t >= 0.0 {
        } else if ready {
            self.button(g, "abrir cofre", cx - 90.0, 292.0, 180.0, 70, "#f3d23c");
        } else {
            let s = secs_to_midnight();
            g.txt(&format!("nuevo cofre en {:02}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60), cx, 312.0, 19.0, "#7a6355", 1, 0.0);
        }
        g.txt("misiones de hoy", cx, 366.0, 22.0, INK, 1, 0.0);
        let mis = self.sv.mis.clone();
        for (i, m) in mis.iter().enumerate() {
            let y = 384.0 + i as f32 * 60.0;
            let done = m.p >= m.n;
            g.fr(ox + 12.0, y, 336.0, 54.0, 12.0, if m.c { "#eef8e4" } else { "#ffffff" }, 2.0);
            g.txt(&mis_text(m), ox + 24.0, y + 16.0, 18.0, INK, 0, 0.0);
            let k = (m.p as f32 / m.n as f32).clamp(0.0, 1.0);
            g.fr(ox + 24.0, y + 32.0, 170.0, 10.0, 5.0, "#efe6d8", 0.0);
            g.fr(ox + 24.0, y + 32.0, 170.0 * k, 10.0, 5.0, "#86d443", 0.0);
            g.stroke(&rr_pts(ox + 24.0, y + 32.0, 170.0, 10.0, 5.0), 1.6, INK, true);
            g.txt(&format!("{}/{}", m.p, m.n), ox + 200.0, y + 38.0, 15.0, "#7a6355", 0, 0.0);
            if m.c {
                g.txt("cobrada", ox + 302.0, y + 28.0, 17.0, "#3c8a1a", 1, 0.0);
            } else if done {
                let bob = (self.time * 6.0).sin() * 1.5;
                g.fr(ox + 262.0, y + 10.0 + bob, 78.0, 34.0, 10.0, "#f3d23c", 2.2);
                g.txt("cobrar", ox + 301.0, y + 27.0 + bob, 18.0, INK, 1, 0.0);
                self.btns.push(Btn { x: ox + 262.0, y: y + 6.0, w: 78.0, h: 42.0, id: 71 + i as u8 });
            } else {
                g.fcirc(ox + 290.0, y + 28.0, 5.0, "#f3d23c", 1.4);
                g.txt(&format!("+{}", m.r), ox + 298.0, y + 29.0, 18.0, INK, 0, 0.0);
            }
        }
        self.button(g, "volver", cx - 70.0, 572.0, 140.0, 74, "#ffffff");
        self.paint_parts(g);
    }
    fn ach_screen(&mut self, g: &mut Painter) {
        let ox = (self.w - BASE_W) / 2.0;
        let cx = self.w / 2.0;
        self.title(g, "logros", ox + 70.0, 46.0, 46.0, -0.04);
        self.medal(g, ox + BASE_W - 82.0, 26.0, 0.6, true);
        g.txt(&format!("{}/{}", self.sv.ach.len(), ACHS.len()), ox + BASE_W - 66.0, 36.0, 24.0, INK, 0, 0.0);
        for (i, a) in ACHS.iter().enumerate() {
            let x = ox + 8.0 + (i % 2) as f32 * 176.0;
            let y = 88.0 + (i / 2) as f32 * 66.0;
            let on = self.sv.ach.iter().any(|x| x == a.0);
            g.fr(x, y, 168.0, 60.0, 12.0, if on { "#fff6c8" } else { "#ffffff" }, 2.0);
            self.medal(g, x + 22.0, y + 22.0, 0.82, on);
            g.txt(a.1, x + 44.0, y + 15.0, 18.0, if on { INK } else { "#7a6355" }, 0, 0.0);
            g.txt(a.2, x + 44.0, y + 33.0, 14.0, "#7a6355", 0, 0.0);
            if on {
                g.txt("¡conseguido!", x + 44.0, y + 49.0, 14.0, "#3c8a1a", 0, 0.0);
            } else {
                let v = self.sv.stat(a.3).min(a.4);
                g.txt(&format!("{}/{}  ·  +{}", v, a.4, a.5), x + 44.0, y + 49.0, 14.0, "#7a6355", 0, 0.0);
            }
        }
        self.button(g, "volver", cx - 70.0, 562.0, 140.0, 6, "#ffffff");
    }
    fn opts_screen(&mut self, g: &mut Painter) {
        let ox = (self.w - BASE_W) / 2.0;
        let cx = self.w / 2.0;
        self.title(g, "opciones", ox + 90.0, 52.0, 46.0, -0.04);
        let bx = cx - 132.0;
        let mut y = 110.0;
        let st = 52.0;
        let yn = |v: bool| if v { "sí" } else { "no" };
        if !self.touch_ui {
            let fs = self.fullscreen;
            self.button(g, &format!("pantalla completa: {}", yn(fs)), bx, y, 264.0, 60, if fs { "#86d443" } else { "#ffffff" });
            y += st;
        }
        let mu = self.sv.music;
        self.button(g, &format!("música: {}", yn(mu)), bx, y, 264.0, 61, if mu { "#86d443" } else { "#ffffff" });
        y += st;
        self.button(g, "restablecer opciones", bx, y, 264.0, 62, "#6cc3ee");
        y += st;
        self.button(g, "borrar progreso", bx, y, 264.0, 63, "#f7b2ae");
        y += st;
        let vy = y + 14.0;
        self.button(g, "volver", cx - 80.0, vy, 160.0, 64, "#ffffff");
        g.txt(&format!("mejor: {}   monedas: {}", self.sv.best(), self.sv.coins), cx, vy + 76.0, 20.0, INK, 1, 0.0);
        let hint = if self.touch_ui { "toca: izquierda · centro dispara · derecha" } else { "flechas o A D mover · espacio disparar · P pausa · F11" };
        g.txt(hint, cx, vy + 104.0, 16.0, "#7a6355", 1, 0.0);
        let look = self.sv.look();
        self.bubu(g, cx, 616.0 + (self.time * 3.0).sin() * 2.0, 1.0, 0.0, false, 0.0, Item::None, &look, 0.0, false, false);
    }
    fn conf_screen(&mut self, g: &mut Painter) {
        let cx = self.w / 2.0;
        self.title(g, "¿borrar todo?", cx, 190.0, 50.0, -0.04);
        g.txt(&format!("se pierde tu récord ({})", self.sv.best()), cx, 258.0, 22.0, INK, 1, 0.0);
        g.txt(&format!("monedas ({}) y compras", self.sv.coins), cx, 288.0, 22.0, INK, 1, 0.0);
        self.button(g, "sí, borrar", cx - 85.0, 340.0, 170.0, 65, "#f7b2ae");
        self.button(g, "cancelar", cx - 85.0, 396.0, 170.0, 66, "#ffffff");
    }

    fn toast_draw(&self, g: &mut Painter) {
        if self.toast.1 > 0.0 {
            g.save();
            g.ga *= (self.toast.1 * 2.0).min(1.0);
            let y = if self.state == State::Shop { 622.0 } else { self.h - 90.0 };
            g.txt(&self.toast.0, self.w / 2.0, y, 22.0, "#e8504a", 1, 5.0);
            g.restore();
        }
    }

    fn pause_screen(&mut self, g: &mut Painter) {
        let cx = self.w / 2.0;
        g.rect(0.0, 0.0, self.w, self.h, Color::new(0.97, 0.94, 0.89, 0.7));
        self.title(g, "pausa", cx, 230.0, 64.0, -0.04);
        self.button(g, "seguir", cx - 80.0, 290.0, 160.0, 11, "#86d443");
        self.button(g, "opciones", cx - 80.0, 346.0, 160.0, 54, "#ffffff");
        self.button(g, "menú", cx - 80.0, 402.0, 160.0, 6, "#ffffff");
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
                self.snd("power");
            }
            6 => self.state = State::Menu,
            7 => {
                self.shop_sel.clear();
                self.state = State::Shop;
            }
            10 => self.state = State::Pause,
            11 => self.state = State::Play,
            12 => self.use_magnet(),
            13 => self.revive(),
            20..=24 => {
                self.shop_tab = (id - 20) as usize;
                self.shop_sel.clear();
            }
            30..=39 => {
                let items = shop_items(TABS[self.shop_tab].0);
                if let Some(it) = items.get((id - 30) as usize) {
                    self.shop_sel = it.0.to_string();
                }
            }
            40 => self.shop_act(),
            50 => {
                self.refresh_day();
                self.chest_t = -1.0;
                self.state = State::Daily;
            }
            51 => self.state = State::Ach,
            52 | 61 => {
                self.sv.music = !self.sv.music;
                self.sv.write();
            }
            53 => {
                self.opt_back = State::Menu;
                self.state = State::Opts;
            }
            54 => {
                self.opt_back = State::Pause;
                self.state = State::Opts;
            }
            60 => {
                self.fullscreen = !self.fullscreen;
                set_fullscreen(self.fullscreen);
            }
            62 => {
                self.sv.music = true;
                self.sv.write();
                self.toast("opciones restablecidas");
            }
            63 => self.state = State::Conf,
            64 => self.state = self.opt_back,
            65 => self.reset_progress(),
            66 => self.state = State::Opts,
            70 => self.open_chest(),
            71..=73 => self.claim_mis((id - 71) as usize),
            74 => {
                self.chest_t = -1.0;
                self.state = State::Menu;
            }
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
                if id != 0 && id != 5 && id != 40 && id != 70 && !(71..=73).contains(&id) {
                    self.snd("click");
                }
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
    let mut au = synth::Audio::new(g.sv.music).await;
    let mut pt = Painter { m: Affine2::IDENTITY, stack: vec![], ga: 1.0, font, px: 1.0 };
    let mut acc = 0.0f32;
    let shot_path = std::env::var("BUBU_SHOT").ok();
    let mut frames = 0u32;

    loop {
        let (sw, sh) = (screen_width(), screen_height());
        let scale = (sw / BASE_W).min(sh / BASE_H);
        let (nw, nh) = (sw / scale, sh / scale);
        // al volver a la app la ventana puede reportar tamaños falsos un instante: se ignoran
        if sw > 100.0 && sh > 100.0 && ((nw - g.w).abs() > 0.5 || (nh - g.h).abs() > 0.5) {
            g.w = nw;
            g.h = nh;
            if matches!(g.state, State::Play | State::Pause | State::Over) {
                g.resize_chains();
            }
        }
        pt.px = 1.0;
        set_default_camera();
        let to_world = |p: Vec2| p / scale;

        let raw_dt = get_frame_time();
        let dt = raw_dt.min(1.0 / 20.0);
        // la app estuvo en segundo plano: pausar en vez de seguir
        if raw_dt > 0.5 && g.state == State::Play {
            g.state = State::Pause;
        }
        g.time += dt;
        g.swap_t = (g.swap_t - dt * 2.0).max(0.0);
        g.toast.1 = (g.toast.1 - dt).max(0.0);
        if g.chest_t >= 0.0 {
            g.chest_t += dt;
        }
        g.update_banners(dt);
        if is_key_pressed(KeyCode::F11) && !g.touch_ui {
            g.act(60);
        }

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
                if is_key_pressed(KeyCode::X) {
                    g.use_magnet();
                }
            }
            State::Pause => {
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Enter) {
                    g.state = State::Play;
                }
            }
            State::Shop | State::Daily | State::Ach => {
                if is_key_pressed(KeyCode::Escape) {
                    g.chest_t = -1.0;
                    g.state = State::Menu;
                }
            }
            State::Opts => {
                if is_key_pressed(KeyCode::Escape) {
                    g.state = g.opt_back;
                }
            }
            State::Conf => {
                if is_key_pressed(KeyCode::Escape) {
                    g.state = State::Opts;
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
            State::Shop => {
                g.paper(&mut pt);
                g.shop_screen(&mut pt);
            }
            State::Daily => {
                g.paper(&mut pt);
                g.daily_screen(&mut pt);
            }
            State::Ach => {
                g.paper(&mut pt);
                g.ach_screen(&mut pt);
            }
            State::Opts => {
                g.paper(&mut pt);
                g.opts_screen(&mut pt);
            }
            State::Conf => {
                g.paper(&mut pt);
                g.conf_screen(&mut pt);
            }
            _ => {
                g.paint_world(&mut pt);
                if g.state == State::Play {
                    g.hud(&mut pt);
                    g.pad(&mut pt);
                }
                g.paint_note(&mut pt);
                if g.state == State::Pause {
                    g.pause_screen(&mut pt);
                }
                if g.state == State::Over {
                    g.over_screen(&mut pt);
                }
            }
        }

        g.toast_draw(&mut pt);
        g.paint_banner(&mut pt);

        // ---- sonido ----
        au.music_on = g.sv.music;
        for n in g.sfx_q.drain(..) {
            au.play(n);
        }
        let playing = matches!(g.state, State::Play);
        let (tt, ff, ii) = (g.wx.ty, g.wx.fade, g.wx.inten);
        let rain = match tt {
            Some(Wx::Rain) | Some(Wx::Acid) => 0.22 * ii * ff,
            Some(Wx::Storm) => 0.32 * ii * ff,
            _ => 0.0,
        };
        let wind = match tt {
            Some(Wx::Wind) => 0.55 * ii * ff,
            Some(Wx::Snow) => 0.12 * ff,
            Some(Wx::Storm) => 0.15 * ff,
            _ => 0.0,
        };
        let (rain, wind) = if playing { (rain, wind) } else { (0.0, 0.0) };
        let motor = if playing { match g.p.pu { Item::Jet => 1, Item::Prop => 2, _ => 0 } } else { 0 };
        let bpm = 116.0 + if g.state != State::Menu { (g.score / 600.0).min(28.0) } else { 0.0 };
        au.update(dt, motor, rain, wind, bpm);
        // captura para pruebas automáticas: BUBU_SHOT=archivo.png [BUBU_STATE=play|over]
        if let Some(path) = &shot_path {
            frames += 1;
            if frames == 2 {
                match std::env::var("BUBU_STATE").as_deref() {
                    Ok("daily") => {
                        g.sv.mis[0].p = g.sv.mis[0].n;
                        g.state = State::Daily;
                    }
                    Ok("ach") => {
                        g.sv.ach = vec!["m500".into(), "c100".into()];
                        g.state = State::Ach;
                    }
                    Ok("opts") => g.state = State::Opts,
                    Ok("shop") => {
                        g.sv.coins = 500;
                        g.state = State::Shop;
                        g.shop_tab = std::env::var("BUBU_TAB").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
                    }
                    Ok("play") | Ok("over") => {
                        g.start();
                        g.score = 6000.0;
                        if let Some(i) = std::env::var("BUBU_WX").ok().and_then(|v| v.parse::<usize>().ok()) {
                            g.bag = vec![WX_ALL[i]];
                            g.wx.next = 0.0;
                            g.p.shield = true;
                            g.p.mag = 5.0;
                            g.sv.eq = vec![("hat".into(), "crown".into()), ("acc".into(), "shades".into()), ("trl".into(), "star".into())];
                        }
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
