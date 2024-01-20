use mqrstt::{
    MqttClient,
    ConnectOptions,
    new_tokio,
    packets::{self, Packet},
    AsyncEventHandler,
    tokio::NetworkStatus,
};
use tokio::time::Duration;
use bytes::Bytes;

pub struct PingPong {
    pub client: MqttClient,
}
impl AsyncEventHandler for PingPong {
    // Handlers only get INCOMING packets. This can change later.
    async fn handle(&mut self, event: packets::Packet) -> () {
        match event {
            Packet::Publish(p) => {
                if let Ok(payload) = String::from_utf8(p.payload.to_vec()) {
                    if payload.to_lowercase().contains("ping") {
                        self.client
                            .publish(
                                p.topic.clone(),
                                p.qos,
                                p.retain,
                                Bytes::from_static(b"pong"),
                            )
                            .await
                            .unwrap();
                        println!("Received Ping, Send pong!");
                    }
                }
            },
            Packet::ConnAck(_) => { println!("Connected!") },
            _ => (),
        }
    }
}

#[tokio::main]
async fn main() {
    let options = ConnectOptions::new("TokioTcpPingPongExample");

    let (mut network, client) = new_tokio(options);

    let stream = tokio::net::TcpStream::connect(("broker.emqx.io", 1883))
        .await
        .unwrap();

    let mut pingpong = PingPong {
        client: client.clone(),
    };

    network.connect(stream, &mut pingpong).await.unwrap();

    client.subscribe("mqrstt").await.unwrap();


    let (n, _) = tokio::join!(
        async {
            loop {
                return match network.poll(&mut pingpong).await {
                    Ok(NetworkStatus::Active) => continue,
                    otherwise => otherwise,
                };
            }
        },
        async {
            tokio::time::sleep(Duration::from_secs(30)).await;
            client.disconnect().await.unwrap();
        }
    );
    assert!(n.is_ok());
}