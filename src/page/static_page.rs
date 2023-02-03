use std::{io::Read, fs::File, path::{Path, PathBuf}};
use bevy::{prelude::*, asset::Asset};
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
pub struct HttpFileServeBundle {
    #[bundle]
    handler: HttpHandlerBundle,
    server: HttpAssetServeComponent,
}

impl HttpFileServeBundle {
    pub fn new(filePath: &Path, servePath: PathBuf, asset_server: &AssetServer) -> Result<Self, std::io::Error> {
        

        Ok(Self {
            handler: HttpHandlerBundle::new(servePath),
            server: HttpAssetServeComponent::new(asset_server.load(filePath))
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
                reply_request_400(&mut reply_events, request);
                continue;
            }

            if let Some(v) = assets.get(&serve.data) {
                let response = Response::new(Body::from(v.data.clone()));
                reply_events.send(HttpRequestReplyEvent::new(Ok(response), request));
            } else {
                reply_request_503(&mut reply_events, request);
            }
        }
    }
}