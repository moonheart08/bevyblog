use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task},
};
use http::{Request, Response};
use hyper::{server::conn::Http, Body};
use log::{debug, error, info, trace, warn};
use std::{error::Error, net::TcpListener, sync::Mutex};
use std::{io::ErrorKind, sync::mpsc};

use crate::{custtcpstream, http::service_adapter::HttpServicer};

#[derive(Component)]
pub(in crate::http) struct HttpRequestComponent {
    pub task: Task<()>,
    pub rxreq: Mutex<mpsc::Receiver<Request<Body>>>,
    pub txres: mpsc::SyncSender<Result<Response<Body>, Box<dyn Error + Send + Sync>>>,
}

#[derive(Bundle)]
struct HttpRequestEntityBundle {
    request: HttpRequestComponent,
    name: Name,
}

#[derive(Resource, Default)]
pub(in crate::http) struct HttpRequestContext {
    listener: Option<TcpListener>,
}

pub(in crate::http) fn http_request_listener_system(mut ctx: ResMut<HttpRequestContext>, mut commands: Commands) {
    if let None = ctx.listener {
        match TcpListener::bind("127.0.0.1:8080") {
            Ok(l) => ctx.listener = Some(l),
            Err(e) => {
                error!("Ran into {e} while trying to set up the listener.");
                return;
            }
        }
        let _ = ctx.listener.as_ref().unwrap().set_nonblocking(true).expect("FUCK");
    }
    let pool = IoTaskPool::get();

    let listener = ctx.listener.as_ref().unwrap();

    while let stream = listener.accept() {
        match stream {
            Ok((s, addr)) => {
                info!("Got a connection from {}", addr);

                let _ = s.set_nonblocking(true);
                let (txreq, rxreq) = mpsc::sync_channel::<Request<Body>>(1);
                let (txres, rxres) = mpsc::sync_channel::<Result<Response<Body>, Box<dyn Error + Send + Sync>>>(1);

                let task = pool.spawn(async move {
                    let stream = custtcpstream::CustTcpStream::new(s);
                    let servicer = HttpServicer::new(txreq, rxres);

                    if let Err(http_err) = Http::new()
                        .http1_only(true)
                        .http1_keep_alive(false)
                        .serve_connection(stream, servicer)
                        .await
                    {
                        error!("Error while serving HTTP connection: {}", http_err);
                    }
                    info!("(ASYNC) Service adapter done (outside serve).");
                });

                let request = HttpRequestComponent {
                    task,
                    rxreq: Mutex::new(rxreq),
                    txres,
                };

                let name = format!("HTTP Request {addr}");

                info!("Spawned request entity as \"{name}\"");

                commands.spawn(HttpRequestEntityBundle {
                    request,
                    name: Name::new(name),
                });
            }
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    return;
                }
                error!("{}", e);
            }
        }
    }
}
