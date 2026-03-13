# oxvoid-css

> Design system. One file: `ox.css`. Prefix: `ox-`. Variables: `--ox-*`.

Agent never writes CSS. Agent uses `ox-*` classes in `el()` and `text_el()` calls.

## Loading

Hosted on Convex, loaded in HTML `<head>`:
```html
<link rel="stylesheet" href="/css/ox" />
```

## Dark mode

```rust
// Set in HTML
attr(&html, "data-theme", "dark");

// Or toggle
on(&toggle, "click", move |_| {
    let html = document().document_element().unwrap();
    let current = html.get_attribute("data-theme").unwrap_or_default();
    let next = if current == "dark" { "light" } else { "dark" };
    html.set_attribute("data-theme", next).unwrap();
});
```

## Pitfalls

### Contrast in dark mode
`ox-text-muted` and `ox-text-subtle` may fail WCAG contrast checks in dark mode. For body text in cards/descriptions, use explicit colors that pass 4.5:1 ratio:
```css
.card-desc { color: #6b7280; }
[data-theme="dark"] .card-desc { color: #b0b4be; }
```

### Load Google Fonts async
Render-blocking font `<link>` costs ~500ms. Use the print/onload swap:
```html
<link href="https://fonts.googleapis.com/css2?family=..." rel="stylesheet" media="print" onload="this.media='all'" />
```

### HTML shell must have lang attribute
```html
<html lang="en" data-theme="dark">
```
Lighthouse flags missing `lang` as an a11y failure.

## Fonts

- **DM Sans** — body text
- **Outfit** — headings (`.ox-font-display`)
- **JetBrains Mono** — code (`.ox-font-mono`)

## Typography

```rust
text_el("h1", "ox-h1", "Page Title");                    // large heading
text_el("h2", "ox-h3 ox-font-display", "Section");       // display font
text_el("p", "ox-text-sm ox-text-muted", "Subtitle");    // muted small text
text_el("code", "ox-code", "let x = 42;");               // inline code
text_el("span", "ox-text-2xl ox-font-bold", "$1,234");   // large bold number
```

Sizes: `.ox-text-{xs,sm,base,lg,xl,2xl,3xl,4xl,5xl,6xl}`
Weight: `.ox-font-{light,normal,medium,semibold,bold,black}`
Align: `.ox-text-{left,center,right}`
Truncate: `.ox-truncate`, `.ox-line-clamp-{2,3}`

## Colors

Text: `.ox-text-{primary,muted,subtle,success,warning,danger,info,white}`
Background: `.ox-bg-{primary,subtle,muted,success,warning,danger,info,white}`

## Layout

```rust
let page = el("div", "ox-container ox-py-8");              // centered container
let grid = el("div", "ox-grid ox-cols-3 ox-gap-4");        // 3-column grid
let row = el("div", "ox-flex ox-gap-2 ox-items-center");   // horizontal flex
let stack = el("div", "ox-stack ox-gap-4");                 // vertical stack
```

Grid: `.ox-grid`, `.ox-cols-{1-6,12}`, `.ox-col-span-{1-12,full}`
Flex: `.ox-flex`, `.ox-stack`, `.ox-items-{start,center,end}`, `.ox-justify-{start,center,end,between}`
Gap: `.ox-gap-{0-8}`

## Spacing

Padding: `.ox-p-{0-12}`, `.ox-px-{0-8}`, `.ox-py-{0-8}`
Margin: `.ox-m-{0-4,auto}`, `.ox-mt-{0-8}`, `.ox-mb-{0-8}`

## Components

### Button
```rust
text_el("button", "ox-btn ox-btn-primary", "Save");
text_el("button", "ox-btn ox-btn-danger ox-btn-sm", "Delete");
text_el("button", "ox-btn ox-btn-ghost", "Cancel");
text_el("button", "ox-btn ox-btn-outline ox-btn-lg", "Learn More");
```
Variants: `primary`, `secondary`, `success`, `danger`, `warning`, `ghost`, `outline`, `link`
Sizes: `sm`, `lg`, `xl`, `icon`, `block`, `pill`

### Card
```rust
let card = el("div", "ox-card ox-card-elevated ox-p-6");
let card = el("div", "ox-card ox-card-hoverable");
// With sections:
let header = el("div", "ox-card-header");
let body = el("div", "ox-card-body");
let footer = el("div", "ox-card-footer");
```

### Form inputs
```rust
let field = el("div", "ox-field");
let label = text_el("label", "ox-label", "Email");
let input = el("input", "ox-input");

// Variants
el("input", "ox-input ox-input-sm");
el("input", "ox-input ox-input-lg");
el("input", "ox-input ox-input-error");
el("select", "ox-select");
el("textarea", "ox-textarea");
```

### Table
```rust
let table = el("table", "ox-table ox-table-striped ox-table-hover");
let thead = el("thead", "");
let tbody = el("tbody", "");
// ox-table-compact for dense data
```

### Badge
```rust
text_el("span", "ox-badge ox-badge-success", "Active");
text_el("span", "ox-badge ox-badge-danger", "Error");
text_el("span", "ox-badge ox-badge-neutral", "Draft");
```

### Alert
```rust
let alert = el("div", "ox-alert ox-alert-warning");
let title = text_el("strong", "ox-alert-title", "Warning");
let msg = text_el("p", "", "This action cannot be undone.");
append(&alert, &[&title, &msg]);
```

### Navbar
```rust
let nav = el("nav", "ox-navbar ox-navbar-sticky");
let brand = text_el("a", "ox-navbar-brand", "My App");
let links = el("div", "ox-navbar-nav");
```

### Tabs
```rust
let tabs = el("div", "ox-tabs");
let tab1 = text_el("button", "ox-tab ox-active", "Overview");
let tab2 = text_el("button", "ox-tab", "Settings");
// Toggle ox-active reactively:
reactive_class(&tab1, "ox-active", move || active.get() == "overview");
```

### Modal
```rust
let overlay = el("div", "ox-modal-overlay");
let modal = el("div", "ox-modal");        // or ox-modal-sm, ox-modal-lg
let header = el("div", "ox-modal-header");
let body = el("div", "ox-modal-body");
let footer = el("div", "ox-modal-footer");
// Toggle with ox-active on overlay:
reactive_class(&overlay, "ox-active", move || is_open.get());
```

### Toast
```rust
let container = el("div", "ox-toast-container");
let toast = el("div", "ox-toast ox-toast-success");
toast.set_text_content(Some("Saved!"));
// Auto-dismiss with setTimeout
```

### Spinner
```rust
let spinner = el("div", "ox-spinner");          // default
let spinner = el("div", "ox-spinner-sm");       // small
let spinner = el("div", "ox-spinner-lg");       // large
```

## Animations

```rust
el("div", "ox-animate-fade-in");
el("div", "ox-animate-fade-up");
el("div", "ox-animate-scale-in ox-delay-200");
```

Options: `fade-in`, `fade-up`, `fade-down`, `scale-in`, `slide-right`, `slide-left`, `bounce`, `pulse`
Delays: `ox-delay-{100,200,300,400,500}`

## Responsive

Grid columns adapt: `.ox-{sm,md,lg}-cols-{2-6}`
Hide: `.ox-hide-sm`, `.ox-hide-below-lg`

```rust
let grid = el("div", "ox-grid ox-cols-1 ox-md-cols-2 ox-lg-cols-4 ox-gap-4");
```

## Utilities

```rust
el("div", "ox-border ox-rounded-lg");
el("div", "ox-shadow-lg");
el("div", "ox-relative");
el("div", "ox-hidden");                        // display: none
el("div", "ox-sr-only");                       // screen reader only
el("hr", "ox-divider");
```
