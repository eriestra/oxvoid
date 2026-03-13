//! ox∅ DOM helpers — thin layer over web-sys. ~50 lines of real logic.

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, Event, HtmlInputElement};

use crate::effect;

/// Get the document.
pub fn document() -> Document {
    web_sys::window()
        .expect("no window")
        .document()
        .expect("no document")
}

/// Create an element with a class string.
pub fn el(tag: &str, class: &str) -> Element {
    let doc = document();
    let e = doc.create_element(tag).expect("create_element failed");
    if !class.is_empty() {
        e.set_class_name(class);
    }
    e
}

/// Create an element with text content and a class string.
pub fn text_el(tag: &str, class: &str, text: &str) -> Element {
    let e = el(tag, class);
    e.set_text_content(Some(text));
    e
}

/// Attach an event listener.
pub fn on(element: &Element, event: &str, handler: impl Fn(Event) + 'static) {
    let cb = Closure::wrap(Box::new(handler) as Box<dyn Fn(Event)>);
    element
        .add_event_listener_with_callback(event, cb.as_ref().unchecked_ref())
        .expect("addEventListener failed");
    cb.forget(); // intentional leak — lives for app lifetime
}

/// Append multiple children to a parent.
pub fn append(parent: &Element, children: &[&Element]) {
    for child in children {
        parent.append_child(child).expect("appendChild failed");
    }
}

/// Set a static attribute.
pub fn attr(element: &Element, key: &str, value: &str) {
    element.set_attribute(key, value).expect("setAttribute failed");
}

/// Reactively toggle display:none based on a condition.
pub fn show_when(element: &Element, condition: impl Fn() -> bool + 'static) {
    let el = element.clone();
    effect(move || {
        if condition() {
            el.remove_attribute("style").ok();
        } else {
            el.set_attribute("style", "display:none").unwrap();
        }
    });
}

/// Reactively set an attribute from a closure.
pub fn reactive_attr(element: &Element, key: &str, value_fn: impl Fn() -> String + 'static) {
    let el = element.clone();
    let key = key.to_string();
    effect(move || {
        el.set_attribute(&key, &value_fn()).unwrap();
    });
}

/// Reactively toggle a CSS class based on a condition.
pub fn reactive_class(element: &Element, class: &str, condition: impl Fn() -> bool + 'static) {
    let el = element.clone();
    let class = class.to_string();
    effect(move || {
        let list = el.class_list();
        if condition() {
            list.add_1(&class).unwrap();
        } else {
            list.remove_1(&class).unwrap();
        }
    });
}

/// Extract the string value from an input event target.
pub fn input_value(event: &Event) -> String {
    event
        .target()
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
}

/// Log to browser console.
pub fn log(msg: &str) {
    web_sys::console::log_1(&msg.into());
}
