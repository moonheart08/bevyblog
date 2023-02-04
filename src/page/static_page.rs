use std::{path::{Path, PathBuf}};
use bevy::prelude::*;
use http::{Response, Method};
use hyper::Body;

use crate::{http::events::HttpRequestReplyEvent, page::error_replies::reply_request_500};

use super::{pathspec::{HttpHandlerBundle, HttpHandlerRequestMailbox, HttpHandlerPathSpec}, error_replies::{reply_request_400, reply_request_503}, assets::WebFileAsset};

/// A very simple server that replies to GET requests with pre-loaded data.
#[derive(Component)]
pub struct HttpAssetServeComponent {
    data: Handle<WebFileAsset>,
    mimetype: &'static str,
}

impl HttpAssetServeComponent {
    pub fn new(data: Handle<WebFileAsset>, mimetype: &'static str) -> Self {
        Self {
            data,
            mimetype
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
        trace!("Building a new asset server, mapping {file_path:?} to URI {serve_path:?}");
        let mimetype = file_path
            .extension()
            .and_then(|v| v.to_str())
            .and_then(|v| mime_guess::from_ext(v).first_raw())
            .unwrap_or(mime_guess::mime::APPLICATION_OCTET_STREAM.essence_str());

        Ok(Self {
            name: Name::new(format!("Asset Server `{serve_path:?}`")),
            handler: HttpHandlerBundle::new(serve_path),
            server: HttpAssetServeComponent::new(asset_server.load(file_path), mimetype)
        })
    }
}

pub(in super) fn http_string_serve_system(
    //TODO: Bevy gets very upset if I make this filter by changed mailbox, though I do see why.
    mut waiting_requests: Query<(&HttpAssetServeComponent, &mut HttpHandlerRequestMailbox, &HttpHandlerPathSpec)>,
    mut reply_events: EventWriter<HttpRequestReplyEvent>,
    assets: Res<Assets<WebFileAsset>>,
) {
    for (serve, mut mailbox, pathspec) in waiting_requests.iter_mut() {

        while let Some((request, body)) = mailbox.read_message() {
            if body.method() != Method::GET {
                reply_request_400(&mut reply_events, body, request);
                continue;
            }

            if let Some(v) = assets.get(&serve.data) {
                let response = Response::builder()
                    .header("Content-Type", serve.mimetype)
                    .body(Body::from(v.data.clone()));
                
                if let Err(e) = response {
                    error!("Got error while trying to serve {:?}, error is: {}", pathspec.path(), e);
                    reply_request_500(&mut reply_events, body, request);
                } else {
                    reply_events.send(HttpRequestReplyEvent::new(Ok(response.unwrap()), request));
                }
            } else {
                reply_request_503(&mut reply_events, body, request);
            }
        }
    }
}