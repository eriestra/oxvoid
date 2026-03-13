//! ox∅ signal runtime — fine-grained reactivity.
//!
//! No frameworks, no proc macros. Just signals, memos, effects, and batch.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

// ── Runtime globals (thread-local, single-threaded WASM) ──────────

thread_local! {
    static CURRENT_EFFECT: RefCell<Option<usize>> = RefCell::new(None);
    static BATCH_DEPTH: RefCell<u32> = RefCell::new(0);
    static PENDING: RefCell<Vec<usize>> = RefCell::new(Vec::new());
    static EFFECTS: RefCell<std::collections::HashMap<usize, Rc<EffectInner>>> =
        RefCell::new(std::collections::HashMap::new());
}

static NEXT_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
fn next_id() -> usize { NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed) }

// ── Signal ────────────────────────────────────────────────────────

struct SignalInner<T> {
    value: T,
    subscribers: HashSet<usize>,
}

#[derive(Clone)]
pub struct ReadSignal<T: 'static> {
    inner: Rc<RefCell<SignalInner<T>>>,
}

#[derive(Clone)]
pub struct WriteSignal<T: 'static> {
    inner: Rc<RefCell<SignalInner<T>>>,
}

impl<T: Clone + 'static> ReadSignal<T> {
    pub fn get(&self) -> T {
        // Track dependency — borrow_mut briefly, then drop
        let effect_id = CURRENT_EFFECT.with(|ce| *ce.borrow());
        if let Some(id) = effect_id {
            self.inner.borrow_mut().subscribers.insert(id);
        }
        // Read value — separate borrow
        self.inner.borrow().value.clone()
    }

    pub fn get_untracked(&self) -> T {
        self.inner.borrow().value.clone()
    }
}

impl<T: Clone + 'static> WriteSignal<T> {
    pub fn set(&self, value: T) {
        // Set value and collect subscribers — then drop borrow
        let subs = {
            let mut inner = self.inner.borrow_mut();
            inner.value = value;
            inner.subscribers.clone()
        };
        // Notify outside of borrow
        notify(&subs);
    }

    pub fn update(&self, f: impl FnOnce(&mut T)) {
        let subs = {
            let mut inner = self.inner.borrow_mut();
            f(&mut inner.value);
            inner.subscribers.clone()
        };
        notify(&subs);
    }
}

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

struct EffectInner {
    id: usize,
    f: Box<dyn Fn()>,
}

impl EffectInner {
    fn run(&self) {
        let prev = CURRENT_EFFECT.with(|ce| ce.borrow_mut().replace(self.id));
        (self.f)();
        CURRENT_EFFECT.with(|ce| *ce.borrow_mut() = prev);
    }
}

pub fn effect(f: impl Fn() + 'static) {
    let id = next_id();
    let inner = Rc::new(EffectInner { id, f: Box::new(f) });
    EFFECTS.with(|effects| effects.borrow_mut().insert(id, inner.clone()));
    inner.run();
}

// ── Memo ──────────────────────────────────────────────────────────

pub fn memo<T: Clone + 'static>(f: impl Fn() -> T + 'static) -> ReadSignal<T> {
    let (read, write) = signal(f());
    effect(move || write.set(f()));
    read
}

// ── Batch ─────────────────────────────────────────────────────────

pub fn batch(f: impl FnOnce()) {
    BATCH_DEPTH.with(|bd| *bd.borrow_mut() += 1);
    f();
    let should_flush = BATCH_DEPTH.with(|bd| {
        *bd.borrow_mut() -= 1;
        *bd.borrow() == 0
    });
    if should_flush {
        flush_pending();
    }
}

// ── Internals ─────────────────────────────────────────────────────

fn notify(subscribers: &HashSet<usize>) {
    if subscribers.is_empty() { return; }

    let is_batching = BATCH_DEPTH.with(|bd| *bd.borrow() > 0);

    if is_batching {
        PENDING.with(|p| {
            let mut p = p.borrow_mut();
            for &id in subscribers {
                p.push(id);
            }
        });
        return;
    }

    // Collect effects to run — don't hold EFFECTS borrow while running
    let to_run: Vec<Rc<EffectInner>> = EFFECTS.with(|effects| {
        let effects = effects.borrow();
        subscribers.iter()
            .filter_map(|id| effects.get(id).cloned())
            .collect()
    });

    for eff in to_run {
        eff.run();
    }
}

fn flush_pending() {
    let pending = PENDING.with(|p| std::mem::take(&mut *p.borrow_mut()));
    let mut seen = HashSet::new();

    let to_run: Vec<Rc<EffectInner>> = EFFECTS.with(|effects| {
        let effects = effects.borrow();
        pending.into_iter()
            .filter(|id| seen.insert(*id))
            .filter_map(|id| effects.get(&id).cloned())
            .collect()
    });

    for eff in to_run {
        eff.run();
    }
}
