use bevy::prelude::*;

use super::{pathspec::{PathSpecSearcherResource, http_request_sorter_system}, static_page::http_string_serve_system, assets::{WebFileAsset, WebFileLoader, SiteMapAsset}};

/// Provides HTTP page handling, automatically routing requests to any entities with the correct pathspec and mailbox.
/// To receive routed requests, utilize the HttpHandlerBundle and read new requests from your HttpHandlerRequestMailbox component.
#[derive(Default)]
pub struct HttpPageHandlerPlugin {}

impl Plugin for HttpPageHandlerPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(PathSpecSearcherResource::default())
            .add_system(http_request_sorter_system)
            .add_system(http_string_serve_system)
            .add_asset::<WebFileAsset>()
            .add_asset::<SiteMapAsset>()
            .add_asset_loader(WebFileLoader());
    }
}