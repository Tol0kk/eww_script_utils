use std::time::Duration;

use surge_ping::{Config, Client, PingIdentifier, PingSequence};

pub async fn ping() -> Result<(surge_ping::IcmpPacket, Duration), surge_ping::SurgeError> {
    let ip = tokio::net::lookup_host(format!("google.fr:0"))
    .await?
    .next()
    .map(|val| val.ip()).unwrap();

    let config = Config::new();

    let payload = vec![0; 56];
    let client = Client::new(&config).unwrap();
    let mut pinger = client.pinger(ip, PingIdentifier(111)).await;
    pinger.ping(PingSequence(0), &payload).await
}

pub async fn is_connected_to_internet() -> bool {
    match ping().await {
        Ok(_) => true,
        Err(_) => false,
    }
}