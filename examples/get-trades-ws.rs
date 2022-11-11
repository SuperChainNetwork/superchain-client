use std::str::FromStr;

// A lot of crates that you might need are reexported from `superchain-client`
// Checkout the `[dev-dependencies]` section for deps that you might have to include manually
use superchain_client::{
    config::Config,
    ethers::types::H160,
    futures::{self, StreamExt},
    tokio_tungstenite::connect_async,
    tungstenite::{client::IntoClientRequest, http::header::AUTHORIZATION},
    QueryOptions, WsClient,
};

/// The list of pairs we want to receive event for
/// An empty list, or `None` means all pairs
const PAIRS_FILTER: [&str; 1] = ["0xb4e16d0168e52d35cacd2c6185b44281ec28c9dc"];
/// The block height we want to receive prices from
const FROM_BLOCK: Option<u64> = Some(15_645_429);
/// the block height we want to receive prices to (inclusive)
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

    // Then we tell the WsClient that we want uniswap v2 prices
    let pairs = PAIRS_FILTER
        .iter()
        .map(|pair| H160::from_str(pair).unwrap());
    let stream = client
        .get_trades(
            pairs,
            QueryOptions::default()
                .with_start(FROM_BLOCK)
                .with_end(TO_BLOCK_INC),
        )
        .await
        .unwrap();
    futures::pin_mut!(stream);

    // And that's it! Now we can stream prices:
    while let Some(res) = stream.next().await {
        let trade = res.unwrap();
        println!("{trade:?}");
    }
}
