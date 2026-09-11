pub mod crypto_feed;
pub mod index_feed;
pub mod options_feed;
pub mod primary_fx;
pub mod publish_queue;
pub mod reconnect;
pub mod secondary_fx;
pub mod tradingview;

pub fn set_ws_nodelay(
    stream: &tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) {
    use tokio_tungstenite::MaybeTlsStream;
    let res = match stream.get_ref() {
        MaybeTlsStream::Plain(tcp) => tcp.set_nodelay(true),
        MaybeTlsStream::Rustls(tls) => tls.get_ref().0.set_nodelay(true),
        _ => Ok(()),
    };
    if let Err(err) = res {
        tracing::trace!(error = %err, "failed to set TCP_NODELAY on client websocket");
    }
}
