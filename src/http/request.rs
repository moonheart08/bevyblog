use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task, ComputeTaskPool}, app::AppExit,
};
use http::{Request, Response};
use hyper::{server::conn::Http, Body};
use log::{error, info};
use std::{error::Error, net::TcpListener, sync::Mutex};
use std::{io::ErrorKind, sync::mpsc};

use crate::{custtcpstream, http::service_adapter::HttpSingleServicer};

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

pub(in crate::http) fn http_request_listener_system(mut ctx: ResMut<HttpRequestContext>, mut commands: Commands, mut exit: EventWriter<AppExit>) {
    if let None = ctx.listener {
        match TcpListener::bind("0.0.0.0:8080") {
            Ok(l) => ctx.listener = Some(l),
            Err(e) => {
                error!("Ran into {e} while trying to set up the listener. Cannot continue.");
                exit.send(AppExit::default()); // Exit.
                return;
            }
        }

        ctx.listener.as_ref().unwrap().set_nonblocking(true).expect("Nonblocking unavailable, can't continue!");
    }
    let pool = ComputeTaskPool::get();

    let listener = ctx.listener.as_ref().unwrap();

    loop {
        let stream = listener.accept();
        match stream {
            Ok((s, addr)) => {
                info!("Got a connection from {}", addr);

                let _ = s.set_nonblocking(true);
                let (txreq, rxreq) = mpsc::sync_channel::<Request<Body>>(1);
                let (txres, rxres) = mpsc::sync_channel::<Result<Response<Body>, Box<dyn Error + Send + Sync>>>(1);

                let task = pool.spawn(async move {
                    let stream = custtcpstream::CustTcpStream::new(s);
                    let servicer = HttpSingleServicer::new(txreq, rxres);

                    if let Err(http_err) = Http::new()
                        .http1_keep_alive(false)
                        .http2_keep_alive_interval(None)
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
