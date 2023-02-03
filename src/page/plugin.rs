use bevy::prelude::*;

use super::{pathspec::{PathSpecSearcherResource, http_request_sorter_system}, static_page::http_string_serve_system};


#[derive(Default)]
pub struct HttpPageHandlerPlugin {}

impl Plugin for HttpPageHandlerPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(PathSpecSearcherResource::default())
            .add_system(http_request_sorter_system)
            .add_system(http_string_serve_system);
    }
}