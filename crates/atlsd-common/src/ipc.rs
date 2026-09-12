use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::broadcast;
use tracing::{error, info};

/// High-throughput, non-blocking Unix Domain Socket broadcaster for Hot-Path IPC.
pub struct UdsBroadcaster {
    path: PathBuf,
    tx: broadcast::Sender<Arc<[u8]>>,
}

impl UdsBroadcaster {
    pub async fn bind<P: AsRef<Path>>(path: P) -> std::io::Result<Arc<Self>> {
        let path = path.as_ref().to_path_buf();
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let listener = UnixListener::bind(&path)?;

        // Ensure permissive socket permissions so containers running as other users can connect
        if let Ok(metadata) = std::fs::metadata(&path) {
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o666);
            let _ = std::fs::set_permissions(&path, permissions);
        }

        let (tx, _) = broadcast::channel::<Arc<[u8]>>(4096);

        let broadcaster = Arc::new(Self {
            path: path.clone(),
            tx: tx.clone(),
        });

        let server_broadcaster = broadcaster.clone();
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((mut stream, _)) => {
                        let mut rx = server_broadcaster.tx.subscribe();
                        tokio::spawn(async move {
                            while let Ok(msg) = rx.recv().await {
                                let len = (msg.len() as u32).to_be_bytes();
                                if stream.write_all(&len).await.is_err() {
                                    break;
                                }
                                if stream.write_all(&msg).await.is_err() {
                                    break;
                                }
                            }
                        });
                    }
                    Err(err) => {
                        error!(error = %err, "uds listener accept error");
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                }
            }
        });

        info!(path = ?path, "uds broadcaster bound and listening");
        Ok(broadcaster)
    }

    pub fn broadcast(&self, payload: &[u8]) -> usize {
        let arc_msg: Arc<[u8]> = Arc::from(payload);
        self.tx.send(arc_msg).unwrap_or(0)
    }

    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl Drop for UdsBroadcaster {
    fn drop(&mut self) {
        if self.path.exists() {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

/// Client receiver for reading length-prefixed frames from a UdsBroadcaster socket.
pub struct UdsReceiver {
    stream: UnixStream,
}

impl UdsReceiver {
    pub async fn connect<P: AsRef<Path>>(path: P, timeout: Duration) -> std::io::Result<Self> {
        let stream = tokio::time::timeout(timeout, UnixStream::connect(path.as_ref()))
            .await
            .map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::TimedOut, "uds connect timeout")
            })??;
        Ok(Self { stream })
    }

    pub async fn recv(&mut self) -> std::io::Result<Vec<u8>> {
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        self.stream.read_exact(&mut buf).await?;
        Ok(buf)
    }
}
