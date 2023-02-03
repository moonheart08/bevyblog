use log::{error, info};
use std::{
    error::Error,
    fmt::Display,
    sync::{Arc, Mutex},
};

use bevy::prelude::*;
use http::{Request, Response};
use hyper::Body;

use super::request::HttpRequestComponent;

#[derive(Debug)]
pub struct HttpRequestReceivedEvent {
    pub body: Arc<Request<Body>>,
    pub ent: Entity,
}

#[derive(Debug)]
pub struct HttpRequestReplyEvent {
    body: Mutex<Result<Response<Body>, Box<dyn Error + Send + Sync>>>,
    ent: Entity,
}

impl HttpRequestReplyEvent {
    pub fn new(result: Result<Response<Body>, Box<dyn Error + Send + Sync>>, request: Entity) -> Self{
        HttpRequestReplyEvent {
            body: Mutex::new(result),
            ent: request,
        }
    }
}

pub(in crate::http) fn http_request_events_system(
    req_comp: Query<(Entity, &HttpRequestComponent)>,
    mut recv_ev_writer: EventWriter<HttpRequestReceivedEvent>,
    mut reply_ev_reader: EventReader<HttpRequestReplyEvent>,
) {
    for (ent, comp) in req_comp.iter() {
        let l = comp.rxreq.lock().unwrap();

        if let Ok(body) = l.try_recv() {
            recv_ev_writer.send(HttpRequestReceivedEvent {
                body: Arc::new(body),
                ent,
            });
            info!("Sent off received event.");
        }
    }

    for i in reply_ev_reader.iter() {
        if let Ok((_, comp)) = req_comp.get(i.ent) {
            info!("Got a reply, trying to send it!");
            let mut bodylock = i.body.lock().unwrap();
            let mut err: Result<Response<Body>, Box<dyn Error + Send + Sync>> = Err(Box::new(TakenError()));
            std::mem::swap(&mut *bodylock, &mut err);
            if comp.task.is_finished() || comp.txres.try_send(err).is_err() {
                error!(
                    "Tried to reply to a request with id {:?} after it's already done.",
                    i.ent
                );

                continue;
            }
        }
    }
}

pub(in crate::http) fn http_finalizer(req_comp: Query<(Entity, &HttpRequestComponent, &Name)>, mut cmds: Commands) {
    for (e, comp, name) in req_comp.iter() {
        if comp.task.is_finished() {
            cmds.entity(e).despawn();
            info!("Finalizing \"{}\"", name.as_str());
        }
    }
}

pub(in crate::http) fn http_hello_world_system(
    mut recv_ev_reader: EventReader<HttpRequestReceivedEvent>,
    mut reply_ev_writer: EventWriter<HttpRequestReplyEvent>,
) {
    for ev in recv_ev_reader.iter() {
        info!("Got received event, replying hello world.");
        reply_ev_writer.send(HttpRequestReplyEvent {
            body: Mutex::new(Ok(Response::new(Body::from("Hello, World!")))),
            ent: ev.ent,
        })
    }
}

#[derive(Debug)]
pub struct TakenError();
impl Display for TakenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[taken data]")
    }
}

impl Error for TakenError {}
