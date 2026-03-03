use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::{Context, Layer};

#[derive(Eq, PartialEq)]
struct CachedEvent {
    level: Level,
    message: Option<String>,
}

impl tracing::field::Visit for CachedEvent {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_string());
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn Debug) {
        if field.name() == "message" && self.message.is_none() {
            self.message = Some(format!("{value:?}"));
        }
    }
}

impl CachedEvent {
    fn new(event: &Event<'_>) -> Self {
        let mut s = Self {
            level: event.metadata().level().to_owned(),
            message: None,
        };

        event.record(&mut s);
        s.message.get_or_insert(event.metadata().name().to_string());

        s
    }
}

impl Hash for CachedEvent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.message.hash(state);
        self.level.hash(state);
    }
}

impl CachedEvent {
    fn message(&self) -> String {
        self.message
            .clone()
            .expect("Event had no name and no message field")
    }
}

struct State {
    window_start: Instant,
    window_size: Duration,
    repititions: HashMap<CachedEvent, usize>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            window_start: Instant::now(),
            window_size: Duration::from_millis(300),
            repititions: HashMap::default(),
        }
    }
}

impl State {
    fn count(&mut self, event: &Event<'_>) -> bool {
        let event = CachedEvent::new(event);

        let repeated = self
            .repititions
            .entry(event)
            .and_modify(|x| *x += 1)
            .or_insert(0);

        *repeated == 0
    }

    fn next_window(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.window_start) > self.window_size {
            self.window_start = now;
            return true;
        }

        false
    }

    fn flush(&mut self) -> HashMap<CachedEvent, usize> {
        let mut repititions = HashMap::default();
        std::mem::swap(&mut repititions, &mut self.repititions);

        repititions.retain(|_, c| *c > 0);
        repititions
    }
}

pub struct ThrottleLayer<L> {
    inner: L,
    state: Mutex<State>,
}

impl<L> ThrottleLayer<L> {
    pub fn new(inner: L) -> Self {
        Self {
            inner,
            state: Mutex::default(),
        }
    }
}

impl<L> Drop for ThrottleLayer<L> {
    fn drop(&mut self) {
        let mut st = self.state.lock().expect("lock");
        st.flush();
    }
}

impl<S, L> Layer<S> for ThrottleLayer<L>
where
    S: Subscriber,
    L: Layer<S>,
{
    fn enabled(&self, metadata: &tracing::Metadata<'_>, ctx: Context<'_, S>) -> bool {
        self.inner.enabled(metadata, ctx)
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let mut st = self.state.lock().expect("lock");

        let mut repititions = HashMap::default();
        if st.next_window() {
            repititions = st.flush();
        }

        let should_forward = st.count(event);
        drop(st);

        for (rep_event, count) in repititions.drain() {
            match rep_event.level {
                Level::ERROR => tracing::error!("[repeated x{count}] {}", rep_event.message()),
                Level::WARN => tracing::warn!("[repeated x{count}] {}", rep_event.message()),
                Level::INFO => tracing::info!("[repeated x{count}] {}", rep_event.message()),
                Level::DEBUG => tracing::debug!("[repeated x{count}] {}", rep_event.message()),
                Level::TRACE => tracing::trace!("[repeated x{count}] {}", rep_event.message()),
            }
        }

        if should_forward {
            self.inner.on_event(event, ctx);
        }
    }

    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::span::Id,
        ctx: Context<'_, S>,
    ) {
        self.inner.on_new_span(attrs, id, ctx)
    }

    fn on_record(
        &self,
        id: &tracing::span::Id,
        values: &tracing::span::Record<'_>,
        ctx: Context<'_, S>,
    ) {
        self.inner.on_record(id, values, ctx)
    }

    fn on_enter(&self, id: &tracing::span::Id, ctx: Context<'_, S>) {
        self.inner.on_enter(id, ctx)
    }

    fn on_exit(&self, id: &tracing::span::Id, ctx: Context<'_, S>) {
        self.inner.on_exit(id, ctx)
    }

    fn on_close(&self, id: tracing::span::Id, ctx: Context<'_, S>) {
        self.inner.on_close(id, ctx)
    }

    fn on_id_change(&self, old: &tracing::span::Id, new: &tracing::span::Id, ctx: Context<'_, S>) {
        self.inner.on_id_change(old, new, ctx)
    }

    fn on_follows_from(
        &self,
        span: &tracing::span::Id,
        follows: &tracing::span::Id,
        ctx: Context<'_, S>,
    ) {
        self.inner.on_follows_from(span, follows, ctx)
    }
}
