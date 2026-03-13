# ox∅ (oxvoid)

> Agent-native WASM apps. No framework. Describe it, it's live.

## What it is

Three Rust files, one CSS file, skills.

```
src/signals.rs    ~150 lines   reactive runtime
src/convex.rs     ~80 lines    HTTP calls to Convex
src/dom.rs        ~50 lines    DOM helpers
ox.css            one file     design system
skills/           markdown     agent knowledge
```

Dependencies: `wasm-bindgen`, `web-sys`, `js-sys`.

Output: ~20KB gzipped WASM. Sub-second rebuilds.

## Why

Frameworks carry human bias. Leptos is SolidJS in Rust — good core, growing bloat. Next.js energy.

ox∅ has no opinions. Signals are ~150 lines you own. DOM is `web-sys` direct. Convex is four HTTP calls. CSS is a file. Skills are markdown.

The agent reads skills, writes plain Rust, builds WASM. No framework to learn, no abstraction to fight, no magic to debug.

## Quick start

```sh
# prerequisites: rustup target add wasm32-unknown-unknown && cargo install wasm-bindgen-cli
sh build.sh
# serve dist/ + index.html + ox.css with any static server
```

## Build

```sh
sh build.sh                  # → dist/oxvoid.wasm + dist/oxvoid.js
sh publish.sh <slug>         # build + upload to Convex → live URL
```

## Stack

| Layer | What | Size |
|---|---|---|
| Reactivity | `signals.rs` | ~150 lines |
| DOM | `web-sys` (direct) | ~50 lines helpers |
| Backend | Convex (raw HTTP) | ~80 lines |
| Styling | `ox.css` | one file |
| Knowledge | `skills/*.md` | markdown |
| Build | `cargo` + `wasm-bindgen` | two commands |

## License

MIT
