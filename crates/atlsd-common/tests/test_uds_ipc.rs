use atlsd_common::ipc::{UdsBroadcaster, UdsReceiver};
use std::time::Duration;

#[tokio::test]
async fn test_uds_broadcaster_and_receiver() {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let socket_path = format!("/tmp/test_uds_{}_{}.sock", std::process::id(), ts);
    let broadcaster = UdsBroadcaster::bind(&socket_path).await.unwrap();
    let mut receiver = UdsReceiver::connect(&socket_path, Duration::from_secs(1))
        .await
        .unwrap();

    // Wait briefly for the server accept task to register the subscriber
    for _ in 0..50 {
        if broadcaster.subscriber_count() > 0 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    let test_msg = r#"{"symbol":"XAUUSD","price":4414.50}"#;
    broadcaster.broadcast(test_msg.as_bytes());

    let received = tokio::time::timeout(Duration::from_millis(500), receiver.recv())
        .await
        .expect("timeout")
        .expect("stream error");

    assert_eq!(std::str::from_utf8(&received).unwrap(), test_msg);
    let _ = std::fs::remove_file(&socket_path);
}
