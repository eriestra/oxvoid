# oxvoid-signals

> Reactive runtime. ~150 lines. The core of ox∅.

## Import

```rust
use oxvoid::{signal, memo, effect, batch};
```

## signal(initial) → (ReadSignal<T>, WriteSignal<T>)

Creates a reactive value. Reading inside an `effect` or `memo` tracks it as a dependency.

```rust
let (count, set_count) = signal(0i32);

// Read
count.get()              // tracks dependency
count.get_untracked()    // reads without tracking

// Write
set_count.set(5)         // replace
set_count.update(|n| *n += 1)  // mutate in place
```

**Types:** any `T: Clone + 'static`. Common: `i32`, `f64`, `String`, `bool`, `Vec<T>`, `Option<T>`.

```rust
let (name, set_name) = signal(String::new());
let (items, set_items) = signal(Vec::<Item>::new());
let (selected, set_selected) = signal::<Option<u64>>(None);
let (visible, set_visible) = signal(false);
```

## memo(fn) → ReadSignal<T>

Derived value. Recalculates only when dependencies change. Use for computed state.

```rust
let (price, set_price) = signal(100.0f64);
let (qty, set_qty) = signal(3i32);

let total = memo(move || price.get() * qty.get() as f64);
let formatted = memo(move || format!("${:.2}", total.get()));

total.get()      // 300.0 — recalculates only when price or qty changes
formatted.get()  // "$300.00"
```

**Chain memos freely:**
```rust
let a = memo(move || count.get() * 2);
let b = memo(move || a.get() + 10);
let c = memo(move || if b.get() > 20 { "high" } else { "low" });
```

## effect(fn)

Side effect. Runs immediately, re-runs when dependencies change. Use for DOM updates.

```rust
// Basic — updates DOM when count changes
effect({
    let label = label.clone();
    move || label.set_text_content(Some(&count.get().to_string()))
});

// Multiple dependencies
effect(move || {
    let msg = format!("{} x {} = {}", price.get(), qty.get(), total.get());
    log(&msg);
});
```

**Clone pattern:** clone DOM elements before the `move` closure. The closure owns the clone.

```rust
let display = text_el("span", "ox-text-xl", "0");
effect({
    let display = display.clone();  // clone here
    move || {                       // move owns the clone
        display.set_text_content(Some(&count.get().to_string()));
    }
});
```

## batch(fn)

Group multiple signal updates. Effects run once after all sets, not after each.

```rust
batch(|| {
    set_x.set(10);
    set_y.set(20);
    set_z.set(30);
}); // effects that depend on x, y, z run once
```

**When to use:** when updating multiple related signals in one user action (form submit, data load, etc).

## Patterns

### Toggle
```rust
let (open, set_open) = signal(false);
on(&btn, "click", move |_| set_open.update(|v| *v = !*v));
show_when(&panel, move || open.get());
```

### List state
```rust
let (todos, set_todos) = signal(Vec::<Todo>::new());

// Add
set_todos.update(|list| list.push(Todo { id: next_id(), text, done: false }));

// Remove
set_todos.update(|list| list.retain(|t| t.id != id));

// Toggle
set_todos.update(|list| {
    if let Some(t) = list.iter_mut().find(|t| t.id == id) {
        t.done = !t.done;
    }
});
```

### Form state
```rust
let (name, set_name) = signal(String::new());
let (email, set_email) = signal(String::new());

let is_valid = memo(move || !name.get().is_empty() && email.get().contains('@'));

on(&name_input, "input", move |e| set_name.set(input_value(&e)));
on(&email_input, "input", move |e| set_email.set(input_value(&e)));
on(&submit_btn, "click", move |_| {
    if is_valid.get() {
        // submit
    }
});
```

### Loading state
```rust
let (loading, set_loading) = signal(false);
let (data, set_data) = signal::<Option<Vec<Item>>>(None);
let (error, set_error) = signal::<Option<String>>(None);

// Fetch
set_loading.set(true);
match convex_query(URL, "items:list", json!({})).await {
    Ok(val) => { set_data.set(Some(parse(val))); set_error.set(None); }
    Err(e) => { set_error.set(Some(e.to_string())); }
}
set_loading.set(false);
```
