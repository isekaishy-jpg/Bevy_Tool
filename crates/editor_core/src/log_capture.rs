use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::log::{tracing, tracing_subscriber, BoxedLayer};
use bevy::prelude::Resource;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

const DEFAULT_MAX_ENTRIES: usize = 2000;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp_unix_ms: u64,
    pub level: tracing::Level,
    pub target: String,
    pub message: String,
}

#[derive(Debug)]
struct LogBufferInner {
    entries: VecDeque<LogEntry>,
}

#[derive(Resource, Debug, Clone)]
pub struct LogBuffer {
    inner: Arc<Mutex<LogBufferInner>>,
    max_entries: usize,
}

impl Default for LogBuffer {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(LogBufferInner {
                entries: VecDeque::new(),
            })),
            max_entries: DEFAULT_MAX_ENTRIES,
        }
    }
}

impl LogBuffer {
    pub fn snapshot(&self) -> Vec<LogEntry> {
        let Ok(guard) = self.inner.lock() else {
            return Vec::new();
        };
        guard.entries.iter().cloned().collect()
    }

    pub fn len(&self) -> usize {
        let Ok(guard) = self.inner.lock() else {
            return 0;
        };
        guard.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub fn log_capture_layer(app: &mut bevy::prelude::App) -> Option<BoxedLayer> {
    let buffer = LogBuffer::default();
    let inner = buffer.inner.clone();
    let max_entries = buffer.max_entries;
    app.insert_resource(buffer);
    Some(Box::new(LogCaptureLayer { inner, max_entries }))
}

struct LogCaptureLayer {
    inner: Arc<Mutex<LogBufferInner>>,
    max_entries: usize,
}

impl<S> Layer<S> for LogCaptureLayer
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = LogVisitor::default();
        event.record(&mut visitor);

        let message = if let Some(message) = visitor.message {
            message
        } else if !visitor.fields.is_empty() {
            visitor.fields.join(" ")
        } else {
            "<no message>".to_string()
        };

        let entry = LogEntry {
            timestamp_unix_ms: now_unix_ms(),
            level: *event.metadata().level(),
            target: event.metadata().target().to_string(),
            message,
        };

        let Ok(mut guard) = self.inner.lock() else {
            return;
        };
        guard.entries.push_back(entry);
        while guard.entries.len() > self.max_entries {
            guard.entries.pop_front();
        }
    }
}

#[derive(Default)]
struct LogVisitor {
    message: Option<String>,
    fields: Vec<String>,
}

impl tracing::field::Visit for LogVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{value:?}"));
        } else {
            self.fields.push(format!("{}={value:?}", field.name()));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_string());
        } else {
            self.fields.push(format!("{}=\"{}\"", field.name(), value));
        }
    }
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}
