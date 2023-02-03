use std::{path::{PathBuf, Path}, sync::Arc, ffi::OsStr, collections::HashMap};

use bevy::prelude::*;
use http::{Request};
use hyper::Body;
use log::{debug, error, info, trace, warn};

use crate::http::events::{HttpRequestReceivedEvent, HttpRequestReplyEvent};

use super::error_replies::reply_request_404;

#[derive(Component)]
pub struct HttpHandlerPathSpec {
    path: PathBuf,
}

#[derive(Component)]
pub struct HttpHandlerRequestMailbox {
    mailbox: Vec<(Entity, Arc<Request<Body>>)>,
}

impl HttpHandlerRequestMailbox {
    pub fn new() -> Self {
        HttpHandlerRequestMailbox {
            mailbox: Vec::default(),
        }
    }

    pub fn read_message(&mut self) -> Option<(Entity, Arc<Request<Body>>)> {
        self.mailbox.pop()
    }

    pub fn push_message(&mut self, handler: Entity, msg: Arc<Request<Body>>) {
        self.mailbox.push((handler, msg));
    }
}

#[derive(Bundle)]
pub struct HttpHandlerBundle {
    spec: HttpHandlerPathSpec,
    mailbox: HttpHandlerRequestMailbox,
}

impl HttpHandlerBundle {
    pub fn new(path: PathBuf) -> Self {
        HttpHandlerBundle {
            spec: HttpHandlerPathSpec { path },
            mailbox: HttpHandlerRequestMailbox::new(),
        }
    }
}

pub fn check_path_matches(check: &Path, pattern: &Path) -> bool {
    if check.iter().count() < pattern.iter().count() {
        return false;
    }

    let mut pattern_iter = pattern.iter();
    for segment in check.iter() {
        let pattern_next = pattern_iter.next();
        if let Some(pattern) = pattern_next {
            if pattern == OsStr::new("*") {
                return true; // Glob, so we match everything from here on.
            }
            
            if segment != pattern {
                return false;
            }
        } else {
            return false;
        }
    }

    return true;
}
 
#[derive(Resource, Default)]
pub(in super) struct PathSpecSearcherResource {
    path_set: HashMap<Entity, PathBuf> // Wiring up the logic to have an acceleration structure, and then not actually doing it. Classic.
}

pub(in super) fn http_request_sorter_system(
    modified_path_specs: Query<(Entity, &HttpHandlerPathSpec, Changed<HttpHandlerPathSpec>)>,
    mut path_mailboxes: Query<(Entity, &mut HttpHandlerRequestMailbox)>,
    mut events: EventReader<HttpRequestReceivedEvent>,
    mut reply_events: EventWriter<HttpRequestReplyEvent>,
    mut searcher: ResMut<PathSpecSearcherResource>
) {
    for (entity, spec, _) in modified_path_specs.iter() {
        searcher.path_set.insert(entity, spec.path.clone());
    }

    'outer: 
    for ev in events.iter() {
        let path = PathBuf::from(ev.body.uri().path());
        for (k, pattern) in searcher.path_set.iter() {
            if !check_path_matches(&path, pattern) {
                continue;
            }

            if let Ok((_, mut mailbox)) = path_mailboxes.get_mut(k.to_owned()) {
                mailbox.push_message(ev.ent, ev.body.clone());
                continue 'outer; // Move to the next event, don't fall through!
            }
        }

        warn!("Failed to find any handler for {path:?}. 404ing.");
        reply_request_404(&mut reply_events, ev.ent);
    }
}
