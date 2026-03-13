//! ox∅ signal runtime — fine-grained reactivity in ~150 lines.
//!
//! No frameworks, no proc macros. Just signals, memos, effects, and batch.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

// ── Runtime globals (thread-local, single-threaded WASM) ──────────

thread_local! {
    /// Currently executing effect (if any). Effects push themselves here
    /// so that signal reads can register as dependencies.
    static CURRENT_EFFECT: RefCell<Option<Rc<EffectInner>>> = RefCell::new(None);

    /// Batch depth counter. When > 0, effects are deferred.
    static BATCH_DEPTH: RefCell<u32> = RefCell::new(0);

    /// Pending effects to run after batch completes.
    static PENDING: RefCell<Vec<Rc<EffectInner>>> = RefCell::new(Vec::new());
}

// ── Signal ────────────────────────────────────────────────────────

/// Shared signal state.
struct SignalInner<T> {
    value: T,
    subscribers: HashSet<usize>,
}

/// Readable signal handle. Clone is cheap (Rc).
#[derive(Clone)]
pub struct ReadSignal<T: 'static> {
    inner: Rc<RefCell<SignalInner<T>>>,
}

/// Writable signal handle. Clone is cheap (Rc).
#[derive(Clone)]
pub struct WriteSignal<T: 'static> {
    inner: Rc<RefCell<SignalInner<T>>>,
}

impl<T: Clone + 'static> ReadSignal<T> {
    /// Read the current value, tracking this signal as a dependency of
    /// the currently running effect/memo.
    pub fn get(&self) -> T {
        // Register dependency
        CURRENT_EFFECT.with(|ce| {
            if let Some(eff) = ce.borrow().as_ref() {
                self.inner.borrow_mut().subscribers.insert(eff.id);
            }
        });
        self.inner.borrow().value.clone()
    }

    /// Read without tracking (no dependency registered).
    pub fn get_untracked(&self) -> T {
        self.inner.borrow().value.clone()
    }
}

impl<T: Clone + 'static> WriteSignal<T> {
    /// Replace the value and notify subscribers.
    pub fn set(&self, value: T) {
        self.inner.borrow_mut().value = value;
        notify(&self.inner.borrow().subscribers);
    }

    /// Update the value in place and notify subscribers.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        f(&mut self.inner.borrow_mut().value);
        let subs = self.inner.borrow().subscribers.clone();
        notify(&subs);
    }
}

/// Create a reactive signal.
pub fn signal<T: Clone + 'static>(initial: T) -> (ReadSignal<T>, WriteSignal<T>) {
    let inner = Rc::new(RefCell::new(SignalInner {
        value: initial,
        subscribers: HashSet::new(),
    }));
    (
        ReadSignal { inner: inner.clone() },
        WriteSignal { inner },
    )
}

// ── Effect ────────────────────────────────────────────────────────

static NEXT_EFFECT_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

struct EffectInner {
    id: usize,
    f: RefCell<Box<dyn Fn()>>,
}

impl EffectInner {
    fn run(self: &Rc<Self>) {
        // Push this effect as the current tracker
        let prev = CURRENT_EFFECT.with(|ce| ce.borrow_mut().replace(self.clone()));
        // Run the effect function
        (self.f.borrow())();
        // Restore previous tracker
        CURRENT_EFFECT.with(|ce| *ce.borrow_mut() = prev);
    }
}

impl std::hash::Hash for EffectInner {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.id.hash(state); }
}
impl PartialEq for EffectInner {
    fn eq(&self, other: &Self) -> bool { self.id == other.id }
}
impl Eq for EffectInner {}

/// All live effects, keyed by ID.
thread_local! {
    static EFFECTS: RefCell<std::collections::HashMap<usize, Rc<EffectInner>>> =
        RefCell::new(std::collections::HashMap::new());
}

/// Create a reactive effect. Runs immediately, re-runs when dependencies change.
pub fn effect(f: impl Fn() + 'static) {
    let id = NEXT_EFFECT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let inner = Rc::new(EffectInner {
        id,
        f: RefCell::new(Box::new(f)),
    });
    EFFECTS.with(|effects| effects.borrow_mut().insert(id, inner.clone()));
    inner.run();
}

// ── Memo ──────────────────────────────────────────────────────────

/// Create a derived/computed signal. Recalculates only when dependencies change.
pub fn memo<T: Clone + 'static>(f: impl Fn() -> T + 'static) -> ReadSignal<T> {
    let (read, write) = signal(f());
    effect(move || {
        let new_val = f();
        write.set(new_val);
    });
    read
}

// ── Batch ─────────────────────────────────────────────────────────

/// Group multiple signal updates. Effects run once after all sets complete.
pub fn batch(f: impl FnOnce()) {
    BATCH_DEPTH.with(|bd| *bd.borrow_mut() += 1);
    f();
    BATCH_DEPTH.with(|bd| {
        *bd.borrow_mut() -= 1;
        if *bd.borrow() == 0 {
            flush_pending();
        }
    });
}

// ── Internals ─────────────────────────────────────────────────────

fn notify(subscribers: &HashSet<usize>) {
    let is_batching = BATCH_DEPTH.with(|bd| *bd.borrow() > 0);

    EFFECTS.with(|effects| {
        let effects = effects.borrow();
        for &id in subscribers {
            if let Some(eff) = effects.get(&id) {
                if is_batching {
                    PENDING.with(|p| p.borrow_mut().push(eff.clone()));
                } else {
                    eff.run();
                }
            }
        }
    });
}

fn flush_pending() {
    let pending = PENDING.with(|p| std::mem::take(&mut *p.borrow_mut()));
    // Deduplicate by ID
    let mut seen = HashSet::new();
    for eff in pending {
        if seen.insert(eff.id) {
            eff.run();
        }
    }
}
