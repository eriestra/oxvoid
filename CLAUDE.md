# CLAUDE.md

ox∅ (oxvoid) — agent-native WASM apps. No framework. One file per app.

> Read the relevant skill file BEFORE writing code. Skills are the source of truth.

## What ox∅ is

A Rust crate (~280 lines) + a CSS file. That's it.

- `src/signals.rs` — reactive runtime (signal, memo, effect, batch)
- `src/convex.rs` — four HTTP functions + one WebSocket
- `src/dom.rs` — thin helpers over web-sys
- `ox.css` — design system (hosted on Convex)

Dependencies: `wasm-bindgen`, `web-sys`, `js-sys`. Three crates. No frameworks.

## How agents use it

1. Read the relevant skill file(s)
2. Write **one `.rs` file** — the entire app
3. `sh build.sh` → WASM
4. `sh publish.sh <slug>` → live

Agent never touches framework code. Agent writes one file that imports `oxvoid::*`.

## Skills Index

```
skills/oxvoid-signals.md   — signal, memo, effect, batch
skills/oxvoid-dom.md       — el, text_el, on, append, show_when, keyed_list
skills/oxvoid-convex.md    — convex_query, convex_mutate, convex_action, convex_subscribe
skills/oxvoid-css.md       — ox-* classes, --ox-* variables, components
```

## Workflow

```
read skills → write src/app/<slug>.rs → sh build.sh → sh publish.sh <slug> → live URL
```

## Commands

```sh
sh build.sh              # cargo build + wasm-bindgen → dist/
sh publish.sh <slug>     # build + upload to Convex → live
```

## Conventions

- One file per app: `src/app/<slug>.rs`
- `use oxvoid::*;` at the top
- `#[wasm_bindgen(start)] pub fn main()` as entry point
- CSS classes via string literals in `el()` / `text_el()` — agent never writes CSS
- Snake_case everywhere
- Brand: `ox∅`
