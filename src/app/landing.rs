//! ox∅ landing page — built with ox∅ itself.
//! One file. No framework. Describes it, it's live.

use crate::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// ── Constants ─────────────────────────────────────────────────────

const TITLE: &str = "ox∅";
const TAGLINE: &str = "Describe it, it's live.";
const SUBTITLE: &str = "Agent-native WASM apps. No framework. ~280 lines of Rust, one CSS file, skills. That's it.";

// ── Entry Point ───────────────────────────────────────────────────


pub fn run() {
    console_error_panic_hook::set_once();
    let doc = document();
    let root = doc.get_element_by_id("app").unwrap();

    // Inject landing-specific styles
    inject_styles(&doc);

    // Theme signal
    let (theme, set_theme) = signal("dark".to_string());

    // HTML shell already has lang="en" data-theme="dark" — don't overwrite

    // Build page
    let page = el("main", "landing");
    attr(&page, "role", "main");
    append(&page, &[
        &navbar(&doc, theme.clone(), set_theme),
        &hero(),
        &thesis(),
        &architecture(),
        &what_you_get(),
        &code_example(),
        &benchmarks(),
        &get_started(),
        &footer(),
    ]);
    root.append_child(&page).unwrap();
}

// ── Navbar ────────────────────────────────────────────────────────

fn navbar(
    doc: &web_sys::Document,
    theme: ReadSignal<String>,
    set_theme: WriteSignal<String>,
) -> web_sys::Element {
    let nav = el("nav", "site-nav");

    let brand = text_el("a", "nav-brand", "ox∅");
    attr(&brand, "href", "#");

    let links = el("div", "ox-flex ox-gap-1 ox-items-center");
    let link_names = ["Spec", "Skills", "GitHub"];
    let link_hrefs = [
        "https://github.com/eriestra/oxvoid/blob/main/spec.md",
        "https://github.com/eriestra/oxvoid/tree/main/skills",
        "https://github.com/eriestra/oxvoid",
    ];
    for i in 0..link_names.len() {
        let a = text_el("a", "nav-link", link_names[i]);
        attr(&a, "href", link_hrefs[i]);
        attr(&a, "target", "_blank");
        links.append_child(&a).unwrap();
    }

    // Theme toggle
    let toggle = text_el("button", "nav-link theme-toggle", "◑");
    let doc_el = doc.document_element().unwrap();
    on(&toggle, "click", move |_| {
        let current = theme.get();
        let next = if current == "dark" { "light".to_string() } else { "dark".to_string() };
        doc_el.set_attribute("data-theme", &next).unwrap();
        set_theme.set(next);
    });
    links.append_child(&toggle).unwrap();

    append(&nav, &[&brand, &links]);
    nav
}

// ── Hero ──────────────────────────────────────────────────────────

fn hero() -> web_sys::Element {
    let section = el("section", "hero");

    let title = text_el("h1", "hero-title", TITLE);
    let tagline = text_el("p", "hero-tagline", TAGLINE);
    let subtitle = text_el("p", "hero-desc", SUBTITLE);

    // Workflow diagram
    let wf = el("div", "wf");
    let steps = [
        ("agent", "wf-dim", "reads skills/"),
        ("→", "wf-arrow", ""),
        ("writes", "wf-hl", "one .rs file"),
        ("→", "wf-arrow", ""),
        ("cargo build", "wf-ok", "WASM"),
        ("→", "wf-arrow", ""),
        ("publish", "wf-ok", "live URL"),
        ("", "wf-time", "< 5 seconds"),
    ];
    for (label, class, sub) in steps {
        if label == "→" {
            let arrow = text_el("span", class, "→");
            wf.append_child(&arrow).unwrap();
        } else if !label.is_empty() {
            let step = el("div", class);
            let l = text_el("span", "wf-label", label);
            step.append_child(&l).unwrap();
            if !sub.is_empty() {
                let s = text_el("span", "wf-sub", sub);
                step.append_child(&s).unwrap();
            }
            wf.append_child(&step).unwrap();
        } else {
            let time = text_el("div", class, sub);
            wf.append_child(&time).unwrap();
        }
    }

    append(&section, &[&title, &tagline, &subtitle, &wf]);
    section
}

// ── Thesis ────────────────────────────────────────────────────────

fn thesis() -> web_sys::Element {
    let section = sect("The Thesis", "Frameworks carry human bias. ox∅ carries none.");

    let grid = el("div", "ox-grid ox-cols-1 ox-md-cols-2 ox-gap-4");

    let cards = [
        (
            "No Framework",
            "Three Rust files. signals.rs (188 lines), dom.rs (103 lines), convex.rs (119 lines). You own every line. No proc macros, no virtual DOM, no opinions.",
        ),
        (
            "No Build Tool",
            "cargo build + wasm-bindgen. Two commands. No Trunk, no Webpack, no esbuild. First build 3-5s, incremental <1s.",
        ),
        (
            "Skills, Not Source",
            "Agents read four markdown files, never source code. The skills ARE the framework knowledge. Same model that made novoid work.",
        ),
        (
            "~20KB Output",
            "WASM binary + JS glue, gzipped. Smaller than most images. No runtime, no GC, no garbage collection pauses. Near-native performance.",
        ),
    ];

    for (title, desc) in cards {
        let card = el("div", "ox-card ox-p-6");
        let h = text_el("h3", "ox-h4 ox-mb-2", title);
        let p = text_el("p", "ox-text-sm card-desc", desc);
        append(&card, &[&h, &p]);
        grid.append_child(&card).unwrap();
    }

    section.append_child(&grid).unwrap();
    section
}

// ── Architecture ──────────────────────────────────────────────────

fn architecture() -> web_sys::Element {
    let section = sect("Architecture", "What's inside ox∅.");

    let pre = el("pre", "arch-block");
    pre.set_inner_html(
r#"<code>oxvoid crate (~280 lines total)
├── signals.rs    reactive runtime
│   signal()  → (ReadSignal, WriteSignal)
│   memo()    → ReadSignal (derived)
│   effect()  → runs when deps change
│   batch()   → group updates
│
├── dom.rs        thin web-sys helpers
│   el()        → Element
│   text_el()   → Element with text
│   on()        → event listener
│   show_when() → reactive display
│   append()    → parent.appendChild
│
├── convex.rs     four HTTP functions
│   convex_query()     → POST /api/query
│   convex_mutate()    → POST /api/mutation
│   convex_action()    → POST /api/action
│   convex_subscribe() → WebSocket /ws
│
└── ox.css        design system (1099 lines)
    ox-btn, ox-card, ox-input, ox-table,
    ox-badge, ox-modal, ox-tabs, ox-toast...</code>"#
    );

    section.append_child(&pre).unwrap();
    section
}

// ── What You Get ──────────────────────────────────────────────────

fn what_you_get() -> web_sys::Element {
    let section = sect("What You Get", "Every ox∅ app ships with:");

    let grid = el("div", "ox-grid ox-cols-1 ox-md-cols-3 ox-gap-4");

    let items = [
        ("WASM Binary", "~10-20KB gzipped. Runs at near-native speed in any browser. No JS framework overhead."),
        ("Convex Backend", "Realtime database, serverless functions, auth. Four HTTP calls, not an SDK."),
        ("ox.css Design System", "25 component groups, dark mode, responsive, animations. One CSS file, ox-* prefix."),
        ("Signal Reactivity", "Fine-grained dependency tracking. Only what changed re-renders. No virtual DOM diffing."),
        ("Sub-second Rebuilds", "Four crate dependencies. Incremental Rust compilation. <1s after first build."),
        ("Agent Skills", "Four markdown files. Agents read skills, generate correct code. Never read source."),
    ];

    for (title, desc) in items {
        let card = el("div", "ox-card ox-card-hoverable ox-p-5");
        let h = text_el("h3", "ox-text-base ox-font-semibold ox-mb-2", title);
        let p = text_el("p", "ox-text-xs card-desc", desc);
        append(&card, &[&h, &p]);
        grid.append_child(&card).unwrap();
    }

    section.append_child(&grid).unwrap();
    section
}

// ── Code Example ──────────────────────────────────────────────────

fn code_example() -> web_sys::Element {
    let section = sect("One File", "An entire app:");

    // Terminal window
    let term = el("div", "term");
    let bar = el("div", "term-bar");
    for color in ["#ff5f57", "#febc2e", "#28c840"] {
        let dot = el("span", "term-dot");
        attr(&dot, "style", &format!("background:{}", color));
        bar.append_child(&dot).unwrap();
    }
    let title = text_el("span", "term-title", "counter.rs");
    bar.append_child(&title).unwrap();

    let body = el("div", "term-body");
    body.set_inner_html(
r#"<pre><code><span class="t-k">use</span> <span class="t-t">oxvoid</span>::*;
<span class="t-k">use</span> <span class="t-t">wasm_bindgen</span>::prelude::*;

<span class="t-a"></span>
<span class="t-k">pub fn</span> <span class="t-f">main</span>() {
    <span class="t-k">let</span> root = document().get_element_by_id(<span class="t-s">"app"</span>).unwrap();

    <span class="t-c">// One signal, one effect, one button. Done.</span>
    <span class="t-k">let</span> (count, set_count) = signal(<span class="t-n">0i32</span>);
    <span class="t-k">let</span> double = memo(<span class="t-k">move</span> || count.get() * <span class="t-n">2</span>);

    <span class="t-k">let</span> display = el(<span class="t-s">"h1"</span>, <span class="t-s">"ox-h1 ox-text-center"</span>);
    effect({
        <span class="t-k">let</span> display = display.clone();
        <span class="t-k">move</span> || display.set_text_content(Some(&count.get().to_string()))
    });

    <span class="t-k">let</span> btn = text_el(<span class="t-s">"button"</span>, <span class="t-s">"ox-btn ox-btn-primary"</span>, <span class="t-s">"+1"</span>);
    on(&btn, <span class="t-s">"click"</span>, <span class="t-k">move</span> |_| set_count.update(|n| *n += <span class="t-n">1</span>));

    append(&root, &[&display, &btn]);
}</code></pre>"#
    );

    append(&term, &[&bar, &body]);

    // Live demo next to it
    let demo = el("div", "demo-box");
    let demo_title = text_el("p", "ox-text-xs card-desc ox-mb-2", "Live demo:");

    let (count, set_count) = signal(0i32);
    let display = text_el("span", "ox-text-4xl ox-font-bold ox-font-mono", "0");
    effect({
        let display = display.clone();
        move || display.set_text_content(Some(&count.get().to_string()))
    });
    let btn = text_el("button", "ox-btn ox-btn-primary ox-btn-lg ox-mt-4", "+1");
    let set_count_inc = set_count.clone();
    on(&btn, "click", move |_| set_count_inc.update(|n| *n += 1));
    let reset = text_el("button", "ox-btn ox-btn-ghost ox-btn-lg ox-mt-2", "Reset");
    let set_count2 = set_count;
    on(&reset, "click", move |_| set_count2.set(0));

    append(&demo, &[&demo_title, &display, &btn, &reset]);

    let row = el("div", "ox-grid ox-cols-1 ox-lg-cols-2 ox-gap-6 ox-items-start");
    append(&row, &[&term, &demo]);
    section.append_child(&row).unwrap();

    section
}

// ── Benchmarks ────────────────────────────────────────────────────

fn benchmarks() -> web_sys::Element {
    let section = sect("Benchmarks", "ox∅ vs the status quo.");

    let grid = el("div", "ox-grid ox-cols-2 ox-md-cols-4 ox-gap-4");

    let stats = [
        ("~20 KB", "total deployed", "vs 621 KB (Next.js)"),
        ("<1s", "incremental rebuild", "vs 5-10s (Leptos)"),
        ("4", "dependencies", "vs 421 MB node_modules"),
        ("280", "lines of framework", "vs 4,141 (novoid JS)"),
    ];

    for (value, label, versus) in stats {
        let card = el("div", "stat-card");
        let v = text_el("div", "stat-value", value);
        let l = text_el("div", "stat-label", label);
        let vs = text_el("div", "stat-versus", versus);
        append(&card, &[&v, &l, &vs]);
        grid.append_child(&card).unwrap();
    }

    section.append_child(&grid).unwrap();

    // Comparison table
    let table_wrap = el("div", "ox-mt-8");
    let table = el("table", "ox-table ox-table-hover");
    let thead = el("thead", "");
    let tr_head = el("tr", "");
    for col in ["", "ox∅ (Rust/WASM)", "novoid (JS)", "Next.js"] {
        let th = text_el("th", "", col);
        tr_head.append_child(&th).unwrap();
    }
    thead.append_child(&tr_head).unwrap();

    let tbody = el("tbody", "");
    let rows = [
        ["Runtime", "None (WASM)", "V8", "V8 + Node"],
        ["Bundle size", "~20KB", "~95KB", "~600KB+"],
        ["Reactivity", "Fine-grained signals", "Fine-grained signals", "Virtual DOM"],
        ["Build tool", "cargo", "esbuild", "Webpack/Turbopack"],
        ["Rebuild time", "<1s", "23ms", "5-10s"],
        ["Dependencies", "4 crates", "0 (vanilla)", "421MB node_modules"],
        ["Type safety", "Compile-time (rustc)", "Runtime", "TypeScript (erasable)"],
        ["GC pauses", "None", "Yes", "Yes"],
    ];

    for row in rows {
        let tr = el("tr", "");
        for (i, cell) in row.iter().enumerate() {
            let td = text_el("td", if i == 0 { "ox-font-medium" } else { "" }, cell);
            tr.append_child(&td).unwrap();
        }
        tbody.append_child(&tr).unwrap();
    }

    append(&table, &[&thead, &tbody]);
    table_wrap.append_child(&table).unwrap();
    section.append_child(&table_wrap).unwrap();

    section
}

// ── Get Started ───────────────────────────────────────────────────

fn get_started() -> web_sys::Element {
    let section = sect("Get Started", "");

    let term = el("div", "term");
    let bar = el("div", "term-bar");
    for color in ["#ff5f57", "#febc2e", "#28c840"] {
        let dot = el("span", "term-dot");
        attr(&dot, "style", &format!("background:{}", color));
        bar.append_child(&dot).unwrap();
    }

    let body = el("div", "term-body");
    body.set_inner_html(
r#"<pre><code><span class="t-c"># prerequisites</span>
<span class="t-p">$</span> rustup target add wasm32-unknown-unknown
<span class="t-p">$</span> cargo install wasm-bindgen-cli

<span class="t-c"># clone and build</span>
<span class="t-p">$</span> git clone https://github.com/eriestra/oxvoid
<span class="t-p">$</span> cd oxvoid
<span class="t-p">$</span> sh build.sh

<span class="t-c"># or just describe what you want</span>
<span class="t-p">$</span> claude

<span class="t-g"># Describe it. It's live.</span></code></pre>"#
    );

    append(&term, &[&bar, &body]);

    let links = el("div", "ox-flex ox-gap-3 ox-mt-6 ox-justify-center");
    let spec_link = text_el("a", "ox-btn ox-btn-primary ox-btn-lg", "Read the Spec");
    attr(&spec_link, "href", "https://github.com/eriestra/oxvoid/blob/main/spec.md");
    attr(&spec_link, "target", "_blank");
    let skills_link = text_el("a", "ox-btn ox-btn-outline ox-btn-lg", "Browse Skills");
    attr(&skills_link, "href", "https://github.com/eriestra/oxvoid/tree/main/skills");
    attr(&skills_link, "target", "_blank");
    append(&links, &[&spec_link, &skills_link]);

    append(&section, &[&term, &links]);
    section
}

// ── Footer ────────────────────────────────────────────────────────

fn footer() -> web_sys::Element {
    let foot = el("footer", "site-footer");
    let text = text_el("p", "ox-text-sm card-desc", "ox∅ — The agent-native runtime.");
    let link = text_el("a", "ox-text-sm ox-text-primary", "GitHub");
    attr(&link, "href", "https://github.com/eriestra/oxvoid");
    attr(&link, "target", "_blank");
    append(&foot, &[&text, &link]);
    foot
}

// ── Helpers ───────────────────────────────────────────────────────

fn sect(title: &str, subtitle: &str) -> web_sys::Element {
    let section = el("section", "sect");
    let label = text_el("span", "sect-label", title);
    let heading = text_el("h2", "sect-heading", subtitle);
    if subtitle.is_empty() {
        append(&section, &[&label]);
    } else {
        append(&section, &[&label, &heading]);
    }
    section
}

fn inject_styles(doc: &web_sys::Document) {
    let style = doc.create_element("style").unwrap();
    style.set_text_content(Some(LANDING_CSS));
    doc.query_selector("head")
        .unwrap()
        .unwrap()
        .append_child(&style)
        .unwrap();
}

// ── Landing-specific CSS (beyond ox.css) ──────────────────────────

const LANDING_CSS: &str = r#"
/* Landing layout */
.landing { max-width: 64rem; margin: 0 auto; padding: 0 1.5rem; }

/* Navbar */
.site-nav {
    position: fixed; top: 0; left: 0; right: 0; z-index: 100;
    display: flex; align-items: center; justify-content: space-between;
    padding: 0.75rem 2rem;
    background: color-mix(in srgb, var(--ox-bg) 85%, transparent);
    backdrop-filter: blur(12px);
    border-bottom: 1px solid var(--ox-border);
}
.nav-brand {
    font-size: 1.5rem; font-weight: 800;
    font-family: var(--ox-font-display);
    background: linear-gradient(135deg, var(--ox-primary-400), var(--ox-primary-600));
    -webkit-background-clip: text; -webkit-text-fill-color: transparent;
}
.nav-link {
    padding: 0.375rem 0.75rem; font-size: 0.875rem; font-weight: 500;
    color: var(--ox-text-muted); border-radius: 0.5rem;
    cursor: pointer; border: none; background: none; font-family: inherit;
    transition: all 150ms ease;
}
.nav-link:hover { color: var(--ox-text); background: var(--ox-bg-muted); }
.theme-toggle { font-size: 1.25rem; }

/* Contrast-safe muted text */
.card-desc { color: #6b7280; }
[data-theme="dark"] .card-desc { color: #b0b4be; }

/* Hero */
.hero {
    text-align: center; padding: 8rem 0 4rem;
}
.hero-title {
    font-size: 6rem; font-weight: 900; letter-spacing: -0.04em;
    font-family: var(--ox-font-display);
    background: linear-gradient(135deg, var(--ox-primary-300), var(--ox-primary-600), #f97316);
    -webkit-background-clip: text; -webkit-text-fill-color: transparent;
    line-height: 1;
    animation: heroGlow 8s ease-in-out infinite alternate;
}
@keyframes heroGlow {
    0% { filter: brightness(1); }
    100% { filter: brightness(1.15); }
}
.hero-tagline {
    font-size: 1.5rem; font-weight: 600; color: var(--ox-text);
    margin: 1rem 0 0.5rem; font-family: var(--ox-font-display);
}
.hero-desc {
    font-size: 1rem; color: var(--ox-gray-500); max-width: 36rem; margin: 0 auto;
    line-height: 1.7;
}
[data-theme="dark"] .hero-desc { color: #b0b4be; }

/* Workflow */
.wf {
    display: flex; align-items: center; justify-content: center;
    gap: 0.5rem; margin-top: 2.5rem; flex-wrap: wrap;
}
.wf-dim, .wf-hl, .wf-ok {
    display: flex; flex-direction: column; align-items: center;
    padding: 0.5rem 1rem; border-radius: 0.5rem;
    font-size: 0.75rem; font-family: var(--ox-font-mono);
}
.wf-dim { background: var(--ox-bg-muted); color: var(--ox-text-muted); }
.wf-hl { background: var(--ox-primary-50); color: var(--ox-primary-600); border: 1px solid var(--ox-primary-200); }
.wf-ok { background: var(--ox-success-50); color: var(--ox-success-700); }
.wf-label { font-weight: 600; }
.wf-sub { font-size: 0.625rem; opacity: 0.7; margin-top: 0.125rem; }
.wf-arrow { color: var(--ox-text-subtle); font-size: 1.25rem; }
.wf-time {
    font-size: 0.75rem; font-weight: 700; color: var(--ox-success-700);
    font-family: var(--ox-font-mono); margin-left: 0.5rem;
}

/* Sections */
.sect { padding: 4rem 0; }
.sect-label {
    display: inline-block; font-size: 0.75rem; font-weight: 700;
    text-transform: uppercase; letter-spacing: 0.1em;
    color: var(--ox-primary-600); margin-bottom: 0.5rem;
}
.sect-heading {
    font-size: 1.5rem; font-weight: 700; color: var(--ox-text);
    font-family: var(--ox-font-display); margin-bottom: 2rem;
}

/* Architecture block */
.arch-block {
    font-family: var(--ox-font-mono); font-size: 0.8rem;
    background: #0a0c10; color: #e4e5e9;
    padding: 1.5rem 2rem; border-radius: 0.75rem;
    overflow-x: auto; line-height: 1.8;
    border: 1px solid var(--ox-border);
}
.arch-block code { font-family: inherit; background: none; color: inherit; }
[data-theme="light"] .arch-block { background: #1a1d27; }

/* Terminal */
.term {
    border-radius: 0.75rem; overflow: hidden;
    border: 1px solid var(--ox-border);
    background: #0a0c10;
}
[data-theme="light"] .term { background: #1a1d27; }
.term-bar {
    display: flex; align-items: center; gap: 0.375rem;
    padding: 0.75rem 1rem; background: rgba(255,255,255,0.05);
    border-bottom: 1px solid rgba(255,255,255,0.05);
}
.term-dot { width: 0.75rem; height: 0.75rem; border-radius: 50%; }
.term-title { font-size: 0.75rem; color: #9ca0ab; margin-left: 0.5rem; font-family: var(--ox-font-mono); }
.term-body { padding: 1.25rem 1.5rem; }
.term-body pre { margin: 0; background: none; }
.term-body code {
    font-family: var(--ox-font-mono); font-size: 0.8rem;
    line-height: 1.8; color: #e4e5e9; background: none;
}
.t-k { color: #c678dd; }
.t-t { color: #e5c07b; }
.t-f { color: #61afef; }
.t-s { color: #98c379; }
.t-n { color: #d19a66; }
.t-c { color: #5c6370; font-style: italic; }
.t-a { color: #e06c75; }
.t-p { color: #61afef; }
.t-g { color: #98c379; font-weight: 600; }

/* Demo box */
.demo-box {
    display: flex; flex-direction: column; align-items: center;
    justify-content: center; padding: 2rem;
    background: var(--ox-bg-subtle); border: 1px solid var(--ox-border);
    border-radius: 0.75rem; min-height: 16rem;
}

/* Stats */
.stat-card {
    text-align: center; padding: 1.5rem;
    background: var(--ox-bg-subtle); border: 1px solid var(--ox-border);
    border-radius: 0.75rem;
}
.stat-value {
    font-size: 2.5rem; font-weight: 900; font-family: var(--ox-font-display);
    color: var(--ox-primary-600); line-height: 1;
}
.stat-label {
    font-size: 0.875rem; font-weight: 600; color: var(--ox-text);
    margin-top: 0.5rem;
}
.stat-versus {
    font-size: 0.75rem; color: var(--ox-gray-500);
    margin-top: 0.25rem; font-family: var(--ox-font-mono);
}
[data-theme="dark"] .stat-versus { color: #9ca0ab; }

/* Footer */
.site-footer {
    display: flex; align-items: center; justify-content: space-between;
    padding: 3rem 0; margin-top: 2rem;
    border-top: 1px solid var(--ox-border);
}

/* Responsive */
@media (max-width: 640px) {
    .hero-title { font-size: 4rem; }
    .hero-tagline { font-size: 1.25rem; }
    .wf { gap: 0.25rem; }
    .wf-arrow { display: none; }
    .site-nav { padding: 0.75rem 1rem; }
    .stat-value { font-size: 1.75rem; }
}
"#;
