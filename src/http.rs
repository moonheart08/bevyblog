use bevy::prelude::*;
use log::{debug, error, info, trace, warn};
pub mod events;
mod request;
mod service_adapter;
use events::*;
use request::*;

#[derive(Default)]
pub struct HttpRequestPlugin {}

impl Plugin for HttpRequestPlugin {
    fn build(&self, app: &mut App) {
        app.world.insert_resource(HttpRequestContext::default());
        app.add_event::<HttpRequestReceivedEvent>()
            .add_event::<HttpRequestReplyEvent>()
            .add_stage_before(
                CoreStage::Update,
                HttpRequestStages::Listener,
                SystemStage::single_threaded(),
            )
            .add_stage_after(
                HttpRequestStages::Listener,
                HttpRequestStages::EventDistro,
                SystemStage::parallel(),
            )
            .add_system_to_stage(HttpRequestStages::Listener, http_request_listener_system)
            .add_system_to_stage(HttpRequestStages::EventDistro, http_request_events_system)
            .add_system(http_hello_world_system)
            .add_system_to_stage(CoreStage::PostUpdate, http_finalizer);
    }
}

fn fuck() {
    info!("E");
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
enum HttpRequestStages {
    /// Handles the actual http listening.
    Listener,
    /// Distributes request handling events.
    EventDistro,
}
