use std::str::FromStr;

// A lot of crates that you might need are reexported from `superchain-client`
// Checkout the `[dev-dependencies]` section for deps that you might have to include manually
use superchain_client::{
    config::Config, ethers::types::H160, futures::StreamExt, tokio_tungstenite::connect_async,
    QueryOptions, WsClient,
};
use tungstenite::{client::IntoClientRequest, http::header::AUTHORIZATION};

/// The list of pairs we want to receive event for
/// An empty list, or `None` means all pairs
const PAIRS_FILTER: [&str; 0] = [];
/// The block height we want to search from
/// `None` means earliest indexed block (usually 0)
const FROM_BLOCK: Option<u64> = Some(15_645_429);
/// The block height we want to search to (inclusive)
/// `None` means continue streaming from head
const TO_BLOCK_INC: Option<u64> = None;
/// The websocket endpoint url
const URL: &str = "wss://beta.superchain.app/websocket";

#[tokio::main]
async fn main() {
    // First, we create a new client
    let mut req = URL.into_client_request().expect("invalid url");
    let config = Config::from_env();
    req.headers_mut().append(
        AUTHORIZATION,
        config
            .get_basic_authorization_value()
            .try_into()
            .expect("invalid auth value"),
    );

    let (websocket, _) = connect_async(req).await.unwrap();
    let client = WsClient::new(websocket).await;

    // Then we tell the WsClient that we want pair created events
    let pairs = PAIRS_FILTER
        .iter()
        .map(|pair| H160::from_str(pair).unwrap());
    let stream = client
        .get_pairs_created(
            pairs,
            QueryOptions::default()
                .with_start(FROM_BLOCK)
                .with_end(TO_BLOCK_INC),
        )
        .await
        .unwrap();
    futures::pin_mut!(stream);

    // And that's it! Now we can stream pairs:
    while let Some(res) = stream.next().await {
        let pair = res.unwrap();
        println!("{pair:?}");
    }
}
