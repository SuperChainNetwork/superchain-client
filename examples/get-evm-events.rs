// A lot of crates that you might need are reexported from `superchain-client`
// Checkout the `[dev-dependencies]` section for deps that you might have to include manually
use superchain_client::{futures::StreamExt, tokio_tungstenite::connect_async, WsClient};

/// The websocket endpoint url
const URL: &str = "ws://localhost:8080/websocket";

#[tokio::main]
async fn main() {
    // First, we create a new client
    // If you need to provide auth headers, you can pass a custom `Request` to `connect_async`
    let (websocket, _) = connect_async(URL).await.unwrap();
    let client = WsClient::new(websocket).await;
    let stream = client.get_evm_events().await.unwrap();

    futures::pin_mut!(stream);

    // And that's it! Now we can stream events:
    while let Some(res) = stream.next().await {
        let event = res.unwrap();
        let event = unsafe { std::str::from_utf8_unchecked(&event) };

        println!("{event:?}");
    }
}
