use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

/// A TCP listener wrapper that automatically enables `TCP_NODELAY` on every
/// accepted incoming connection, bypassing Nagle's 20-40ms buffering delay.
#[derive(Debug)]
pub struct NodelayListener {
    inner: TcpListener,
}

impl NodelayListener {
    pub fn new(inner: TcpListener) -> Self {
        Self { inner }
    }

    pub async fn bind<A: tokio::net::ToSocketAddrs>(addr: A) -> std::io::Result<Self> {
        let inner = TcpListener::bind(addr).await?;
        Ok(Self { inner })
    }

    pub fn inner(&self) -> &TcpListener {
        &self.inner
    }

    pub fn into_inner(self) -> TcpListener {
        self.inner
    }
}

impl axum::serve::Listener for NodelayListener {
    type Io = TcpStream;
    type Addr = SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        loop {
            match self.inner.accept().await {
                Ok((stream, addr)) => {
                    if let Err(err) = stream.set_nodelay(true) {
                        tracing::trace!(error = %err, "failed to set TCP_NODELAY on accepted stream");
                    }
                    return (stream, addr);
                }
                Err(e) => {
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::ConnectionRefused
                            | std::io::ErrorKind::ConnectionAborted
                            | std::io::ErrorKind::ConnectionReset
                    ) {
                        continue;
                    }
                    tracing::error!("accept error: {e}");
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    }

    fn local_addr(&self) -> std::io::Result<Self::Addr> {
        self.inner.local_addr()
    }
}

pub fn tap_nodelay(listener: TcpListener) -> NodelayListener {
    NodelayListener::new(listener)
}
