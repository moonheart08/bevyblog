pub struct HttpHandlerPathSpec {
    path: PathBuf,
}

pub struct HttpHandlerRequestMailbox {
    mailbox: Mutex<Vec<(Entity, Arc<Request<Body>>)>>
}

pub struct HttpHandlerBundle {
    spec: HttpHandlerPathSpec,
    mailbox: HttpHandlerRequestMailbox,
}
impl 