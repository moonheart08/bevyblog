use std::sync::Arc;

use bevy::prelude::*;
use http::{Response, StatusCode, Request};
use log::warn;
use hyper::Body;

use crate::http::events::HttpRequestReplyEvent;

/// Automatically reply to the given request with the 404 (Not Found) page.
pub fn reply_request_404(events: &mut EventWriter<HttpRequestReplyEvent>, request_data: Arc<Request<Body>>, request: Entity) {
    warn!("Replying 404 to request for \"{:?}\".", request_data.uri());
    let mut response = Response::new(Body::from("404 Not Found."));
    let _ = std::mem::replace(response.status_mut(), StatusCode::NOT_FOUND); // why do i have to do it this way.
    events.send(HttpRequestReplyEvent::new(Ok(response), request))
}

/// Automatically reply to the given request with the 400 (Bad Request) page.
pub fn reply_request_400(events: &mut EventWriter<HttpRequestReplyEvent>, request_data: Arc<Request<Body>>, request: Entity) {
    warn!("Replying 400 to request for \"{:?}\".", request_data.uri());
    let mut response = Response::new(Body::from("400 Bad Request."));
    let _ = std::mem::replace(response.status_mut(), StatusCode::BAD_REQUEST); // why do i have to do it this way.
    events.send(HttpRequestReplyEvent::new(Ok(response), request))
}

/// Automatically reply to the given request with the 503 (Service Unavailable) page.
pub fn reply_request_503(events: &mut EventWriter<HttpRequestReplyEvent>, request_data: Arc<Request<Body>>, request: Entity) {
    warn!("Replying 503 to request for \"{:?}\".", request_data.uri());
    let mut response = Response::new(Body::from("503 Service Unavailable."));
    let _ = std::mem::replace(response.status_mut(), StatusCode::SERVICE_UNAVAILABLE); // why do i have to do it this way.
    events.send(HttpRequestReplyEvent::new(Ok(response), request))
}