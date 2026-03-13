# oxvoid-convex

> Convex integration. ~80 lines. Four HTTP functions + one WebSocket.

## Import

```rust
use oxvoid::{convex_query, convex_mutate, convex_action, convex_subscribe};
use serde_json::json;
```

## Constants

```rust
const CONVEX_URL: &str = "https://your-deployment.convex.cloud";
```

## Query (read data)

```rust
let result = convex_query(CONVEX_URL, "tasks:list", json!({ "orgId": org_id })).await;
// → Result<serde_json::Value, ConvexError>

// With no args
let stats = convex_query(CONVEX_URL, "stats:summary", json!({})).await;

// Parse result
if let Ok(val) = result {
    let items: Vec<Item> = serde_json::from_value(val).unwrap();
}
```

## Mutate (write data)

```rust
let result = convex_mutate(CONVEX_URL, "tasks:create", json!({
    "text": "Buy milk",
    "orgId": org_id,
})).await;

// Returns created document ID or result
convex_mutate(CONVEX_URL, "tasks:update", json!({
    "id": task_id,
    "done": true,
})).await;

convex_mutate(CONVEX_URL, "tasks:remove", json!({
    "id": task_id,
})).await;
```

## Action (server-side logic)

```rust
let result = convex_action(CONVEX_URL, "ai:chat", json!({
    "prompt": "Summarize this data",
})).await;

// Actions can call external APIs, run long operations, etc.
convex_action(CONVEX_URL, "reports:export_pdf", json!({
    "id": report_id,
})).await;
```

## Subscribe (live updates)

WebSocket connection. Returns a signal that updates when server data changes.

```rust
let tasks = convex_subscribe(CONVEX_URL, "tasks:list", json!({ "orgId": org_id }));
// → ReadSignal<Option<serde_json::Value>>

// Use in effects — re-runs when data changes on server
effect({
    let list = list_el.clone();
    move || {
        if let Some(data) = tasks.get() {
            // rebuild DOM from fresh data
            let items: Vec<Task> = serde_json::from_value(data).unwrap();
            keyed_list(&list, items, |t| t.id, |task| {
                text_el("li", "ox-p-2", &task.text)
            });
        }
    }
});
```

## Auth headers

For authenticated mutations/actions, pass a secret or token:

```rust
// Mutations that require auth include the secret in args
convex_mutate(CONVEX_URL, "pages:publish", json!({
    "slug": "my-app",
    "html": html_content,
    "secret": PUBLISH_SECRET,
})).await;
```

## Implementation

Each function is a `POST` to `{url}/api/{type}`:

```
POST {CONVEX_URL}/api/query     body: { "path": "tasks:list", "args": { ... } }
POST {CONVEX_URL}/api/mutation   body: { "path": "tasks:create", "args": { ... } }
POST {CONVEX_URL}/api/action     body: { "path": "ai:chat", "args": { ... } }
```

Subscribe opens `WebSocket {CONVEX_URL}/ws` and sends subscription messages.

## Patterns

### Load-once on mount
```rust
wasm_bindgen_futures::spawn_local(async move {
    set_loading.set(true);
    match convex_query(URL, "items:list", json!({})).await {
        Ok(val) => set_items.set(serde_json::from_value(val).unwrap_or_default()),
        Err(e) => set_error.set(Some(e.to_string())),
    }
    set_loading.set(false);
});
```

### Mutate + optimistic update
```rust
on(&btn, "click", move |_| {
    // Optimistic: update UI immediately
    set_items.update(|list| list.push(new_item.clone()));

    // Then sync to server
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = convex_mutate(URL, "items:create", json!({ "text": text })).await {
            // Rollback on error
            set_items.update(|list| list.pop());
            set_error.set(Some(e.to_string()));
        }
    });
});
```

### Live dashboard
```rust
let stats = convex_subscribe(URL, "stats:realtime", json!({}));
let orders = convex_subscribe(URL, "orders:recent", json!({ "limit": 10 }));

effect({
    let revenue_el = revenue_el.clone();
    move || {
        if let Some(data) = stats.get() {
            let rev = data["revenue"].as_f64().unwrap_or(0.0);
            revenue_el.set_text_content(Some(&format!("${:.0}", rev)));
        }
    }
});
```
