//! ox∅ — agent-native WASM apps. No framework.
//!
//! ```rust
//! use oxvoid::*;
//! ```

pub mod signals;
pub mod dom;
pub mod convex;

// Apps — each exposes a `run()` function, no #[wasm_bindgen(start)]
pub mod app {
    pub mod landing;
    pub mod fluid;
    pub mod particles;
    pub mod raymarcher;
    pub mod scene3d;
    pub mod tetris;
    pub mod doom;
    pub mod graphcalc;
}

// Re-export everything at crate root for `use oxvoid::*;`
pub use signals::{signal, memo, effect, batch, ReadSignal, WriteSignal};
pub use dom::{document, el, text_el, on, append, attr, show_when, reactive_attr, reactive_class, input_value, log};
pub use convex::{convex_query, convex_mutate, convex_action, convex_subscribe, ConvexError};

use wasm_bindgen::prelude::*;

/// Entry point — reads `data-app` attribute from `#app` div to decide which app to run.
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    let doc = document();
    let root = doc.get_element_by_id("app").unwrap();
    let app_name = root.get_attribute("data-app").unwrap_or_default();

    match app_name.as_str() {
        "landing" => app::landing::run(),
        "fluid" => app::fluid::run(),
        "particles" => app::particles::run(),
        "raymarcher" => app::raymarcher::run(),
        "scene3d" => app::scene3d::run(),
        "tetris" => app::tetris::run(),
        "doom" => app::doom::run(),
        "graphcalc" => app::graphcalc::run(),
        _ => app::landing::run(),
    }
}
