use std::{
    io::{ErrorKind, Read, Write},
    net::TcpStream,
};
use tokio::io::{AsyncRead, AsyncWrite};

pub(crate) struct CustTcpStream {
    pub(crate) stream: TcpStream,
}

impl CustTcpStream {
    pub fn new(stream: TcpStream) -> Self {
        return Self { stream };
    }
}

impl AsyncRead for CustTcpStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.stream.read(buf.initialize_unfilled()) {
            Ok(v) => {
                buf.advance(v);
                return std::task::Poll::Ready(Ok(()));
            }
            Err(e) => match e.kind() {
                ErrorKind::WouldBlock => return std::task::Poll::Pending,
                _ => return std::task::Poll::Ready(Err(e)),
            },
        }
    }
}

impl AsyncWrite for CustTcpStream {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        match self.stream.write(buf) {
            Ok(v) => {
                return std::task::Poll::Ready(Ok(v));
            }
            Err(e) => match e.kind() {
                ErrorKind::WouldBlock => return std::task::Poll::Pending,
                _ => return std::task::Poll::Ready(Err(e)),
            },
        }
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match self.stream.flush() {
            Ok(_) => {
                return std::task::Poll::Ready(Ok(()));
            }
            Err(e) => match e.kind() {
                ErrorKind::WouldBlock => return std::task::Poll::Pending,
                _ => return std::task::Poll::Ready(Err(e)),
            },
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        return std::task::Poll::Ready(self.stream.shutdown(std::net::Shutdown::Write));
    }
}
