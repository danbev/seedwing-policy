use crate::lang::lir::{EvalContext, TraceHandle, Type};
use crate::runtime::{EvaluationResult, Output};
use crate::value::RuntimeValue;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum MonitorEvent {
    Start(StartEvent),
    Complete(CompleteEvent),
}

impl MonitorEvent {
    pub fn ty(&self) -> Arc<Type> {
        match self {
            MonitorEvent::Start(inner) => inner.ty.clone(),
            MonitorEvent::Complete(inner) => inner.ty.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StartEvent {
    pub correlation: u64,
    pub input: Arc<RuntimeValue>,
    pub ty: Arc<Type>,
}

impl From<StartEvent> for MonitorEvent {
    fn from(event: StartEvent) -> Self {
        MonitorEvent::Start(event)
    }
}

impl From<CompleteEvent> for MonitorEvent {
    fn from(event: CompleteEvent) -> Self {
        MonitorEvent::Complete(event)
    }
}

#[derive(Debug, Clone)]
pub struct CompleteEvent {
    pub correlation: u64,
    pub ty: Arc<Type>,
    pub output: Output,
}

pub struct Monitor {
    correlation: AtomicU64,
    subscribers: Arc<Mutex<Vec<Subscriber>>>,
}

impl Monitor {
    pub fn new() -> Self {
        Self {
            correlation: AtomicU64::new(0),
            subscribers: Arc::new(Default::default()),
        }
    }

    pub async fn subscribe(&self, path: String) -> Receiver<MonitorEvent> {
        let (sender, receiver) = channel(10);
        self.subscribers.lock().await.push(Subscriber {
            path,
            sender: sender,
            disconnected: false,
        });
        receiver
    }

    pub async fn start(&self, input: Arc<RuntimeValue>, ty: Arc<Type>) -> u64 {
        let correlation = self.correlation.fetch_add(1, Ordering::Relaxed);
        let event = StartEvent {
            correlation,
            input,
            ty,
        };
        self.fanout(event.into()).await;
        correlation
    }

    pub async fn complete(&self, correlation: u64, ty: Arc<Type>, output: Output) {
        let event = CompleteEvent {
            correlation,
            ty,
            output,
        };
        self.fanout(event.into()).await;
    }

    async fn fanout(&self, event: MonitorEvent) {
        for subscriber in self
            .subscribers
            .lock()
            .await
            .iter_mut()
            .filter(|sub| sub.interested_in(event.ty()))
        {
            if let Err(err) = subscriber.sender.try_send(event.clone()) {
                match err {
                    TrySendError::Full(_) => {
                        // ehhh
                    }
                    TrySendError::Closed(_) => subscriber.disconnected = true,
                }
            }
        }

        let mut locked = self.subscribers.lock().await;
        let live_subscribers = locked
            .iter()
            .filter(|e| e.disconnected == false)
            .cloned()
            .collect();
        *locked = live_subscribers
    }
}

#[derive(Clone)]
pub struct Subscriber {
    path: String,
    sender: Sender<MonitorEvent>,
    disconnected: bool,
}

impl Subscriber {
    pub fn interested_in(&self, ty: Arc<Type>) -> bool {
        if let Some(name) = ty.name() {
            name.as_type_str().starts_with(&self.path)
        } else {
            false
        }
    }
}