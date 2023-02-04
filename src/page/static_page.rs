use std::{path::{Path, PathBuf}};
use bevy::prelude::*;
use http::{Response, Method};
use hyper::Body;

use crate::http::events::HttpRequestReplyEvent;

use super::{pathspec::{HttpHandlerBundle, HttpHandlerRequestMailbox}, error_replies::{reply_request_400, reply_request_503}, assets::WebFileAsset};

/// A very simple server that replies to GET requests with pre-loaded data.
#[derive(Component)]
pub struct HttpAssetServeComponent {
    data: Handle<WebFileAsset>,
}

impl HttpAssetServeComponent {
    pub fn new(data: Handle<WebFileAsset>) -> Self {
        Self {
            data
        }
    }
}

#[derive(Bundle)]
pub struct HttpAssetServeBundle {
    #[bundle]
    handler: HttpHandlerBundle,
    server: HttpAssetServeComponent,
    name: Name,
}

impl HttpAssetServeBundle {
    pub fn new(file_path: &Path, serve_path: PathBuf, asset_server: &AssetServer) -> Result<Self, std::io::Error> {
        Ok(Self {
            name: Name::new(format!("Asset Server `{serve_path:?}`")),
            handler: HttpHandlerBundle::new(serve_path),
            server: HttpAssetServeComponent::new(asset_server.load(file_path))
        })
    }
}

pub(in super) fn http_string_serve_system(
    //TODO: Bevy gets very upset if I make this filter by changed mailbox, though I do see why.
    mut waiting_requests: Query<(&HttpAssetServeComponent, &mut HttpHandlerRequestMailbox)>,
    mut reply_events: EventWriter<HttpRequestReplyEvent>,
    assets: Res<Assets<WebFileAsset>>,
) {
    for (serve, mut mailbox) in waiting_requests.iter_mut() {
        while let Some((request, body)) = mailbox.read_message() {
            if body.method() != Method::GET {
                reply_request_400(&mut reply_events, body, request);
                continue;
            }

            if let Some(v) = assets.get(&serve.data) {
                let response = Response::new(Body::from(v.data.clone()));
                reply_events.send(HttpRequestReplyEvent::new(Ok(response), request));
            } else {
                reply_request_503(&mut reply_events, body, request);
            }
        }
    }
}