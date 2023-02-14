use crate::ui::LAYOUT_HTML;
use actix::{Actor, StreamHandler};
use actix_web::body::BodyStream;
use actix_web::http::header;
use actix_web::http::header::{HeaderValue, ACCEPT};
use actix_web::Error;
use actix_web::{get, post};
use actix_web::{rt, web, HttpRequest, HttpResponse};
use handlebars::Handlebars;
use seedwing_policy_engine::runtime::monitor::{Monitor, MonitorEvent};
use seedwing_policy_engine::runtime::{Component, Output, TypeName, World};
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex;

const MONITOR_HTML: &str = include_str!("ui/_monitor.html");

#[get("/monitor/{path:.*}")]
pub async fn monitor(
    req: HttpRequest,
    world: web::Data<World>,
    monitor: web::Data<Arc<Mutex<Monitor>>>,
    path: web::Path<String>,
) -> HttpResponse {
    let path = path.replace('/', "::");
    let mut renderer = Handlebars::new();
    renderer.set_prevent_indent(true);
    renderer.register_partial("layout", LAYOUT_HTML).unwrap();
    renderer.register_partial("monitor", MONITOR_HTML).unwrap();

    if let Ok(html) = renderer.render("monitor", &()) {
        HttpResponse::Ok().body(html)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

#[get("/monitor-stream/{path:.*}")]
pub async fn monitor_stream(
    req: HttpRequest,
    monitor_manager: web::Data<Arc<Mutex<Monitor>>>,
    path: web::Path<String>,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let (res, session, msg_stream) = actix_ws::handle(&req, stream)?;
    let path = path.replace('/', "::");
    let receiver = monitor_manager.lock().await.subscribe(path).await;
    // spawn websocket handler (and don't await it) so that the response is returned immediately
    rt::spawn(inner_monitor_stream(session, msg_stream, receiver));

    Ok(res)
}

pub async fn inner_monitor_stream(
    mut session: actix_ws::Session,
    mut msg_stream: actix_ws::MessageStream,
    mut receiver: Receiver<MonitorEvent>,
) {
    loop {
        // todo! listen for close and other failures.
        if let Some(event) = receiver.recv().await {
            if let Ok(event) = Event::try_from(event) {
                if let Ok(json) = serde_json::to_string(&event) {
                    session.text(json).await;
                }
            }
        }
    }
}

#[derive(Serialize)]
pub enum Event {
    Start(Start),
    Complete(Complete),
}

#[derive(Serialize)]
pub struct Start {
    correlation: u64,
    name: Option<String>,
    input: Value,
}

#[derive(Serialize)]
pub struct Complete {
    correlation: u64,
    output: WsOutput,
}

#[derive(Serialize)]
pub enum WsOutput {
    None,
    Identity,
    Transform(Value),
}

impl TryFrom<MonitorEvent> for Event {
    type Error = ();

    fn try_from(value: MonitorEvent) -> Result<Self, Self::Error> {
        match value {
            MonitorEvent::Start(inner) => Ok(Event::Start(Start {
                correlation: inner.correlation,
                name: inner.ty.name().map(|e| e.as_type_str()),
                input: inner.input.as_json(),
            })),
            MonitorEvent::Complete(inner) => Ok(Event::Complete(Complete {
                correlation: inner.correlation,
                output: match inner.output {
                    Output::None => WsOutput::None,
                    Output::Identity => WsOutput::Identity,
                    Output::Transform(val) => WsOutput::Transform(val.as_json()),
                },
            })),
        }
    }
}