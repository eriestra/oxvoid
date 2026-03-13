# ox∅ — spec

> Axiomatic reference. One example per API. Zero tutorials. Zero frameworks.

## Model

ox∅ is a Rust crate (~280 lines) + a CSS file. Agents write **one `.rs` file** per app. Build → publish → live.

```
oxvoid crate (the library)
├── signals.rs    ~150 lines   signal, memo, effect, batch
├── convex.rs     ~80 lines    query, mutate, action, subscribe
└── dom.rs        ~50 lines    el, text_el, show_when, on, keyed_list

ox.css (the design system)
└── hosted on Convex, loaded via <link>

agent writes:
└── src/app/<slug>.rs          one file = entire app
```

---

## App Template

Every app is one file:

```rust
use oxvoid::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    let doc = document();
    let root = doc.get_element_by_id("app").unwrap();

    // signals
    let (count, set_count) = signal(0i32);
    let double = memo(move || count.get() * 2);

    // DOM
    let card = el("div", "ox-card ox-card-elevated ox-p-6 ox-stack ox-gap-4");

    let display = el("h1", "ox-h1 ox-text-center ox-font-mono");
    effect({
        let display = display.clone();
        move || display.set_text_content(Some(&count.get().to_string()))
    });

    let sub = el("p", "ox-text-muted ox-text-center");
    effect({
        let sub = sub.clone();
        move || sub.set_text_content(Some(&format!("double: {}", double.get())))
    });

    let btn = text_el("button", "ox-btn ox-btn-primary", "+1");
    on(&btn, "click", move |_| set_count.update(|n| *n += 1));

    append(&card, &[&display, &sub, &btn]);
    root.append_child(&card).unwrap();
}
```

Served by:
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>counter</title>
    <link rel="stylesheet" href="/css/ox" />
</head>
<body>
    <div id="app"></div>
    <script type="module">
        import init from '/wasm/<slug>.js';
        init();
    </script>
</body>
</html>
```

---

## Signals API

### signal(initial) → (ReadSignal, WriteSignal)
```rust
let (count, set_count) = signal(0i32);
count.get();                    // read (tracks dependency)
set_count.set(5);               // replace value
set_count.update(|n| *n += 1);  // update in place
count.get_untracked();          // read without tracking
```

### memo(fn) → ReadSignal
```rust
let double = memo(move || count.get() * 2);
double.get(); // recalculates only when dependencies change
```

### effect(fn)
```rust
effect(move || {
    log(&format!("count: {}", count.get())); // runs when count changes
});
```

With cleanup:
```rust
effect(move || {
    let id = set_interval(|| log("tick"), 1000);
    on_cleanup(move || clear_interval(id));
});
```

### batch(fn)
```rust
batch(|| {
    set_a.set(1);
    set_b.set(2);
}); // effects run once, not twice
```

---

## DOM API

Direct `web-sys` with thin helpers. No macros, no virtual DOM.

### document() → Document
```rust
let doc = document(); // shorthand for web_sys::window().document()
```

### el(tag, class) → Element
```rust
let card = el("div", "ox-card");
let header = el("header", "ox-navbar ox-navbar-sticky");
```

### text_el(tag, class, text) → Element
```rust
let btn = text_el("button", "ox-btn ox-btn-primary", "Click me");
let title = text_el("h1", "ox-h1", "Hello ox∅");
```

### on(element, event, handler)
```rust
on(&btn, "click", move |_| set_count.update(|n| *n += 1));
on(&input, "input", move |e| {
    let value = input_value(&e);
    set_text.set(value);
});
```

### append(parent, children)
```rust
append(&card, &[&title, &subtitle, &btn]);
```

### show_when(element, condition)
```rust
show_when(&admin_panel, move || user.get().role == "admin");
```

### attr(element, key, value)
```rust
attr(&input, "placeholder", "Enter text...");
attr(&input, "type", "email");
```

### reactive_attr(element, key, signal)
```rust
reactive_attr(&btn, "disabled", move || is_loading.get());
reactive_class(&card, "ox-active", move || is_open.get());
```

### keyed_list(parent, items, key_fn, render_fn)
```rust
keyed_list(&ul, todos, |t| t.id, |todo| {
    let li = text_el("li", "ox-p-2", &todo.text);
    on(&li, "click", move |_| store.toggle(todo.id));
    li
});
```

### input_value(event) → String
```rust
on(&input, "input", move |e| {
    let v = input_value(&e);
    set_name.set(v);
});
```

---

## Convex API

Four functions. Raw HTTP. No SDK.

### convex_query(url, path, args) → Result<Value>
```rust
let tasks = convex_query(CONVEX_URL, "tasks:list", json!({ "orgId": org_id })).await;
```

### convex_mutate(url, path, args) → Result<Value>
```rust
convex_mutate(CONVEX_URL, "tasks:create", json!({ "text": "Buy milk" })).await;
```

### convex_action(url, path, args) → Result<Value>
```rust
convex_action(CONVEX_URL, "ai:chat", json!({ "prompt": "Hello" })).await;
```

### convex_subscribe(url, path, args) → ReadSignal<Option<Value>>
```rust
let tasks = convex_subscribe(CONVEX_URL, "tasks:list", json!({}));
// tasks.get() → Option<Value>, updates live via WebSocket

effect(move || {
    if let Some(data) = tasks.get() {
        // rebuild list
    }
});
```

---

## CSS Reference

One file: `ox.css`. Prefix: `ox-`. Variables: `--ox-*`.

### Variables
`--ox-primary-{50-900}`, `--ox-gray-{50-900}`, `--ox-success|warning|danger|info-{50,500,700}`, `--ox-bg`, `--ox-bg-subtle`, `--ox-bg-muted`, `--ox-text`, `--ox-text-muted`, `--ox-border`, `--ox-ring`, `--ox-font-sans|mono|display`, `--ox-text-{xs-6xl}`, `--ox-space-{0-24}`, `--ox-radius-{none-full}`, `--ox-shadow-{xs-2xl}`.

### Dark mode
`[data-theme="dark"]` or `.ox-dark`.

### Typography
`.ox-h1`–`.ox-h6`, `.ox-text-{xs-6xl}`, `.ox-font-{light,normal,medium,semibold,bold,black}`, `.ox-font-{sans,mono,display}`, `.ox-text-{left,center,right}`, `.ox-truncate`.

### Layout
`.ox-container`, `.ox-grid`, `.ox-cols-{1-6,12}`, `.ox-col-span-{1-12,full}`, `.ox-flex`, `.ox-stack`, `.ox-items-{start,center,end}`, `.ox-justify-{start,center,end,between}`, `.ox-gap-{0-8}`.

### Spacing
`.ox-p-{0-12}`, `.ox-px-{0-8}`, `.ox-py-{0-8}`, `.ox-m-{0-4,auto}`, `.ox-mt-{0-8}`, `.ox-mb-{0-8}`.

### Components
| Component | Classes |
|---|---|
| Button | `.ox-btn` `.ox-btn-{primary,secondary,success,danger,warning,ghost,outline,link}` `.ox-btn-{sm,lg,xl,icon,block,pill}` `.ox-btn-group` |
| Card | `.ox-card` `.ox-card-{elevated,header,body,footer,hoverable}` |
| Form | `.ox-label` `.ox-input` `.ox-select` `.ox-textarea` `.ox-checkbox` `.ox-radio` `.ox-field` `.ox-input-{sm,lg,error,success}` `.ox-toggle` |
| Table | `.ox-table` `.ox-table-{striped,hover,compact}` |
| Badge | `.ox-badge` `.ox-badge-{primary,success,warning,danger,info,neutral}` `.ox-badge-{dot,lg}` |
| Alert | `.ox-alert` `.ox-alert-{info,success,warning,danger}` `.ox-alert-title` |
| Navbar | `.ox-navbar` `.ox-navbar-{brand,nav,link,sticky}` |
| Tabs | `.ox-tabs` `.ox-tab` `.ox-tab-panel` `.ox-tabs-pill` |
| Modal | `.ox-modal-overlay` `.ox-modal` `.ox-modal-{header,title,body,footer,close,sm,lg,xl}` |
| Drawer | `.ox-drawer-overlay` `.ox-drawer` `.ox-drawer-{left,right,header,title,body,footer,close,sm,lg,xl}` |
| Toast | `.ox-toast-container` `.ox-toast` `.ox-toast-{success,danger,warning}` |
| Other | `.ox-spinner{-sm,-lg}`, `.ox-skeleton`, `.ox-avatar{-sm,-lg,-xl}`, `.ox-progress`, `.ox-divider`, `.ox-tag`, `.ox-accordion`, `.ox-code`, `.ox-pre`, `.ox-prose` |

**Active:** `.ox-active` on modals, drawers, dropdowns, tabs.
**Animations:** `.ox-animate-{fade-in,fade-up,fade-down,scale-in,slide-right,slide-left,bounce,pulse}`.
**Responsive:** `.ox-{sm,md,lg}-cols-{2-6}`, `.ox-hide-sm`, `.ox-hide-below-lg`.

---

## Building

```sh
sh build.sh          # cargo build + wasm-bindgen → dist/
sh publish.sh <slug> # build + upload to Convex → live URL
```

First build: 3-5s. Incremental: <1s.

---

## Cargo.toml (per app)

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
oxvoid = { path = "../oxvoid" }  # or from crates.io
wasm-bindgen = "0.2"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
```

Two dependencies. One file. One build command.

---

## Notes

1. **One file per app.** Agent writes `src/app/<slug>.rs`. That's the entire app.
2. **oxvoid is the crate.** `signals.rs`, `convex.rs`, `dom.rs` are library code, not app code.
3. **CSS is hosted.** `ox.css` lives on Convex, loaded via `<link>`. Agent never writes CSS.
4. **~20KB output.** WASM + JS glue, gzipped.
5. **<1s rebuilds.** Two crates total (oxvoid + wasm-bindgen).
6. **No macros.** No `view!`, no `#[component]`. Plain functions.
7. **Skills are the interface.** Agents read `skills/*.md`, never read source.
