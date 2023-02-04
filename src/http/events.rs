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

/// Event that, when raised, contains information about an incoming HTTP request, namely it's body and attached entity.
#[derive(Debug)]
pub struct HttpRequestReceivedEvent {
    pub body: Arc<Request<Body>>,
    pub ent: Entity,
}

/// Event that, when raised, will be handled to reply to the specified HTTP request.
#[derive(Debug)]
pub struct HttpRequestReplyEvent {
    body: Mutex<Result<Response<Body>, Box<dyn Error + Send + Sync>>>,
    ent: Entity,
}

impl HttpRequestReplyEvent {
    /// Constructs a new HttpRequestReplyEvent, given a response and the request to reply to.
    pub fn new(result: Result<Response<Body>, Box<dyn Error + Send + Sync>>, request: Entity) -> Self {
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
            let uri = body.uri();
            info!("Sent off received event for {ent:?} at URI \"{uri:?}\".");
            recv_ev_writer.send(HttpRequestReceivedEvent {
                body: Arc::new(body),
                ent,
            });

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

/// A fallback error that is used should a bug occur in reply handling that causes us to attempt to reply with bad data.
#[derive(Debug)]
pub struct TakenError();

impl Display for TakenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[taken data]")
    }
}

impl Error for TakenError {}
