use http::{Request, Response};
use hyper::{body::Body, service::Service};
use log::info;
use std::fmt::Display;
use std::sync::mpsc;
use std::{
    error::Error,
    future::Future,
    task::{self, Poll},
};

pub struct HttpSingleServicer {
    out: Option<mpsc::SyncSender<Request<Body>>>,
    inp: Option<mpsc::Receiver<Result<Response<Body>, Box<dyn Error + Send + Sync>>>>,
    done: bool,
}

impl HttpSingleServicer {
    pub fn new(
        out: mpsc::SyncSender<Request<Body>>,
        inp: mpsc::Receiver<Result<Response<Body>, Box<dyn Error + Send + Sync>>>,
    ) -> Self {
        info!("(ASYNC) Service adapter spun up.");
        Self {
            out: Some(out),
            inp: Some(inp),
            done: false,
        }
    }
}

pub struct HttpSingleServicerFuture<E, R> {
    inp: mpsc::Receiver<Result<R, E>>,
}

impl<E, R> HttpSingleServicerFuture<E, R> {
    pub fn new(inp: mpsc::Receiver<Result<R, E>>) -> Self {
        HttpSingleServicerFuture { inp }
    }
}

impl<E, R> Future for HttpSingleServicerFuture<E, R> {
    type Output = Result<R, E>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        if let Ok(v) = self.inp.try_recv() {
            info!("(ASYNC) Task pool sending reply.");
            return Poll::Ready(v);
        }

        cx.waker().wake_by_ref();

        return Poll::Pending;
    }
}

impl Service<Request<Body>> for HttpSingleServicer {
    type Response = Response<Body>;

    type Error = Box<dyn Error + Send + Sync>;

    type Future = HttpSingleServicerFuture<Self::Error, Self::Response>;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.done {
            info!("(ASYNC) Service adapter done (inside serve).");
            return Poll::Ready(Err(Box::new(RequestFinalizedError())));
        }
        cx.waker().wake_by_ref();
        return Poll::Ready(Ok(()));
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        self.out
            .as_mut()
            .unwrap()
            .send(req)
            .expect("Welp, someone screwed up big time, channel is already dead.");
        self.done = true;
        return Self::Future::new(self.inp.take().unwrap());
    }
}

#[derive(Debug)]
pub struct RequestFinalizedError();
impl Display for RequestFinalizedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Request finalized.")
    }
}

impl Error for RequestFinalizedError {}
