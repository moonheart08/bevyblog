use std::{io::{BufReader, Read}, fs::File, path::{Path, PathBuf}};
use bevy::prelude::*;
use http::{Response, Method};
use hyper::Body;

use crate::http::events::HttpRequestReplyEvent;

use super::{pathspec::{HttpHandlerBundle, HttpHandlerRequestMailbox}, error_replies::reply_request_400};

#[derive(Component)]
pub struct HttpStringServeComponent {
    data: String,
}

impl HttpStringServeComponent {
    pub fn new(data: String) -> Self {
        Self {
            data
        }
    }
}

#[derive(Bundle)]
pub struct HttpFileServeBundle {
    #[bundle]
    handler: HttpHandlerBundle,
    server: HttpStringServeComponent,
}

impl HttpFileServeBundle {
    pub fn new(filePath: &Path, servePath: PathBuf) -> Result<Self, std::io::Error> {
        let mut file = File::open(filePath)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        Ok(Self {
            handler: HttpHandlerBundle::new(servePath),
            server: HttpStringServeComponent::new(contents)
        })
    }
}

pub(in super) fn http_string_serve_system(
    //TODO: Bevy gets very upset if I make this filter by changed mailbox, though I do see why.
    mut waiting_requests: Query<(&HttpStringServeComponent, &mut HttpHandlerRequestMailbox)>,
    mut reply_events: EventWriter<HttpRequestReplyEvent>,
) {
    for (serve, mut mailbox) in waiting_requests.iter_mut() {
        while let Some((request, body)) = mailbox.read_message() {
            if body.method() != Method::GET {
                reply_request_400(&mut reply_events, request);
                continue;
            }

            let response = Response::new(Body::from(serve.data.clone()));
            reply_events.send(HttpRequestReplyEvent::new(Ok(response), request));
        }
    }
}