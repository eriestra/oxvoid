# oxvoid-dom

> DOM helpers. ~50 lines over `web-sys`. No macros, no virtual DOM.

## Import

```rust
use oxvoid::{document, el, text_el, on, append, show_when, attr, reactive_attr, reactive_class, keyed_list, input_value, log};
```

## Core helpers

### document() → Document
```rust
let doc = document();
```

### el(tag, class) → Element
```rust
let card = el("div", "ox-card ox-p-4");
let row = el("div", "ox-flex ox-gap-2");
let container = el("div", "ox-container ox-py-8");
```

### text_el(tag, class, text) → Element
```rust
let title = text_el("h1", "ox-h1", "Dashboard");
let btn = text_el("button", "ox-btn ox-btn-primary", "Save");
let badge = text_el("span", "ox-badge ox-badge-success", "Active");
```

### append(parent, children)
```rust
append(&card, &[&title, &subtitle, &content]);
append(&row, &[&btn_save, &btn_cancel]);
append(&root, &[&navbar, &main, &footer]);
```

### on(element, event, handler)
```rust
on(&btn, "click", move |_| set_count.update(|n| *n += 1));
on(&input, "input", move |e| set_name.set(input_value(&e)));
on(&form, "submit", move |e| {
    e.prevent_default();
    // handle submit
});
on(&doc, "keydown", move |e| {
    let key = e.unchecked_ref::<web_sys::KeyboardEvent>().key();
    if key == "Escape" { set_open.set(false); }
});
```

### input_value(event) → String
```rust
on(&input, "input", move |e| {
    let value = input_value(&e);
    set_search.set(value);
});
```

## Reactive DOM

### Reactive text (via effect)
```rust
let label = el("span", "ox-text-lg");
effect({
    let label = label.clone();
    move || label.set_text_content(Some(&format!("${:.2}", total.get())))
});
```

### show_when(element, condition)
Toggles `display: none` reactively.
```rust
show_when(&error_alert, move || error.get().is_some());
show_when(&admin_section, move || role.get() == "admin");
show_when(&empty_state, move || items.get().is_empty());
```

### attr(element, key, value)
Static attribute.
```rust
attr(&input, "type", "email");
attr(&input, "placeholder", "Enter email...");
attr(&link, "href", "https://example.com");
attr(&img, "src", "/img/logo.png");
```

### reactive_attr(element, key, signal_fn)
Attribute that updates reactively.
```rust
reactive_attr(&btn, "disabled", move || is_loading.get());
reactive_attr(&input, "value", move || name.get());
```

### reactive_class(element, class, condition)
Toggle a CSS class reactively.
```rust
reactive_class(&tab, "ox-active", move || active_tab.get() == "home");
reactive_class(&card, "ox-card-elevated", move || is_selected.get());
reactive_class(&row, "ox-bg-subtle", move || is_highlighted.get());
```

### keyed_list(parent, items, key_fn, render_fn)
Efficient keyed list rendering. Diffs by key, minimal DOM mutations.
```rust
keyed_list(&tbody, todos, |t| t.id, |todo| {
    let tr = el("tr", "");
    let td_name = text_el("td", "", &todo.text);
    let td_status = text_el("td", "", if todo.done { "Done" } else { "Pending" });
    let td_action = el("td", "");
    let btn = text_el("button", "ox-btn ox-btn-sm ox-btn-ghost", "Toggle");
    let id = todo.id;
    on(&btn, "click", move |_| toggle(id));
    td_action.append_child(&btn).unwrap();
    append(&tr, &[&td_name, &td_status, &td_action]);
    tr
});
```

### log(msg)
```rust
log("app started");
log(&format!("count: {}", count.get()));
```

## Common patterns

### Card with reactive content
```rust
let card = el("div", "ox-card ox-card-elevated ox-p-6");
let title = text_el("h2", "ox-h4", "Revenue");
let value = el("p", "ox-text-4xl ox-font-bold");
effect({
    let value = value.clone();
    move || value.set_text_content(Some(&format!("${}", revenue.get())))
});
let trend = el("span", "ox-text-sm ox-text-success");
effect({
    let trend = trend.clone();
    move || trend.set_text_content(Some(&format!("+{}%", growth.get())))
});
append(&card, &[&title, &value, &trend]);
```

### Input with two-way binding
```rust
let field = el("div", "ox-field");
let label = text_el("label", "ox-label", "Name");
let input = el("input", "ox-input");
attr(&input, "type", "text");
attr(&input, "placeholder", "Enter name");
on(&input, "input", move |e| set_name.set(input_value(&e)));
effect({
    let input = input.clone();
    move || input.unchecked_ref::<web_sys::HtmlInputElement>().set_value(&name.get())
});
append(&field, &[&label, &input]);
```

### Navbar
```rust
let nav = el("nav", "ox-navbar ox-navbar-sticky");
let brand = text_el("a", "ox-navbar-brand", "ox∅ App");
let links = el("div", "ox-navbar-nav");
append(&links, &[
    &text_el("a", "ox-navbar-link", "Home"),
    &text_el("a", "ox-navbar-link", "Settings"),
]);
append(&nav, &[&brand, &links]);
```

### Modal
```rust
let (open, set_open) = signal(false);

let overlay = el("div", "ox-modal-overlay");
let modal = el("div", "ox-modal");
let header = el("div", "ox-modal-header");
let title = text_el("h3", "ox-modal-title", "Confirm");
let close = text_el("button", "ox-modal-close", "×");
on(&close, "click", move |_| set_open.set(false));
append(&header, &[&title, &close]);

let body = el("div", "ox-modal-body");
body.set_text_content(Some("Are you sure?"));

let footer = el("div", "ox-modal-footer");
let confirm = text_el("button", "ox-btn ox-btn-danger", "Delete");
let cancel = text_el("button", "ox-btn ox-btn-ghost", "Cancel");
on(&cancel, "click", move |_| set_open.set(false));
append(&footer, &[&cancel, &confirm]);

append(&modal, &[&header, &body, &footer]);
overlay.append_child(&modal).unwrap();

reactive_class(&overlay, "ox-active", move || open.get());
```
