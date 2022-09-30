use ethers::types::H160;
use futures::{SinkExt, Stream, StreamExt, TryStreamExt};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::mpsc,
};
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

use crate::{
    types::{PairCreated, Price, Reserves},
    Error, Result,
};

type WsMsg = Result<Vec<u8>>;
type OperationMsg = (Operation, mpsc::UnboundedSender<WsMsg>);

/// A Superchain WebSocket client
pub struct Client {
    backend_tx: mpsc::Sender<OperationMsg>,
}

impl Client {
    /// Create a new [`Client`]
    pub async fn new<S>(websocket: WebSocketStream<S>) -> Self
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let (tx, rx) = mpsc::channel(1024);
        tokio::spawn(BackGroundWorker::new(websocket, rx).run());

        Self { backend_tx: tx }
    }

    /// Get the uniswap v2 pair created events for the provided `pairs_filter` within the specified
    /// block range.
    ///
    /// A `pairs_filter` of `[]` or `None` will yield all `PairCreated` events. If one or more pair
    /// hashes are specified, only these events will be returned (if present).
    ///
    /// A `from_block` of `None` will yield from the earliest indexed block (usually 0).
    /// A `to_block_inc` of `None` will lead to a head following stream.
    pub async fn get_pairs_created(
        &self,
        pairs_filter: impl IntoIterator<Item = H160>,
        from_block: Option<u64>,
        to_block_inc: Option<u64>,
    ) -> Result<impl Stream<Item = Result<PairCreated>> + Send> {
        self.request(Operation::GetPairs {
            pairs: pairs_filter.into_iter().map(|pair| pair.0).collect(),
            start: from_block,
            end: to_block_inc,
        })
        .await
    }

    /// Get the uniswap v2 price quotes for the provided `pairs_filter` within the specified
    /// block range.
    ///
    /// A `pairs_filter` of `[]` or `None` will yield price quotes for all pairs. If one or more
    /// pair hashes are specified, only price quotes for these pairs will be returned (if present).
    ///
    /// A `from_block` of `None` will yield from the earliest indexed block (usually 0).
    /// A `to_block_inc` of `None` will lead to a head following stream.
    pub async fn get_prices(
        &self,
        pairs_filter: impl IntoIterator<Item = H160>,
        from_block: Option<u64>,
        to_block_inc: Option<u64>,
    ) -> Result<impl Stream<Item = Result<Price>> + Send> {
        self.request(Operation::GetPrices {
            pairs: pairs_filter.into_iter().map(|pair| pair.0).collect(),
            start: from_block,
            end: to_block_inc,
        })
        .await
    }

    /// Get the reserves v2 price quotes for the provided `pairs_filter` within the specified
    /// block range.
    ///
    /// A `pairs_filter` of `[]` or `None` will yield price quotes for all pairs. If one or more
    /// pair hashes are specified, only price quotes for these pairs will be returned (if present).
    ///
    /// A `from_block` of `None` will yield from the earliest indexed block (usually 0).
    /// A `to_block_inc` of `None` will lead to a head following stream.
    pub async fn get_reserves(
        &self,
        pairs_filter: impl IntoIterator<Item = H160>,
        from_block: Option<u64>,
        to_block_inc: Option<u64>,
    ) -> Result<impl Stream<Item = Result<Reserves>> + Send> {
        self.request(Operation::GetReserves {
            pairs: pairs_filter.into_iter().map(|pair| pair.0).collect(),
            start: from_block,
            end: to_block_inc,
        })
        .await
    }

    pub async fn get_height(&self) -> Result<u64> {
        let stream = self.raw_request(Operation::GetHeight).await?;
        futures::pin_mut!(stream);
        let bytes = stream
            .next()
            .await
            .transpose()?
            .ok_or_else(|| Error::Custom("empty response from websocket".to_owned()))?;
        let bytes: [u8; 8] = TryFrom::try_from(&*bytes)
            .map_err(|_| Error::Custom("failed to collect bytes for height bytes".to_owned()))?;
        Ok(u64::from_ne_bytes(bytes))
    }

    /// Returns a stream of JSON encoded EVM events (decoded transactions and logs)
    /// from the head of the chain.
    pub async fn get_evm_events(
        &self,
    ) -> Result<impl Stream<Item = std::io::Result<Vec<u8>>> + Send> {
        self.request_raw(Operation::GetEvmEvents).await
    }

    async fn request<T>(&self, operation: Operation) -> Result<impl Stream<Item = Result<T>> + Send>
    where
        T: serde::de::DeserializeOwned + 'static,
    {
        let raw_data_stream = self.raw_request(operation).await?.boxed();

        let stream = csv_async::AsyncDeserializer::from_reader(raw_data_stream.into_async_read())
            .into_deserialize()
            .map_err(Error::from)
            .into_stream();

        Ok(stream)
    }

    async fn raw_request(
        &self,
        operation: Operation,
    ) -> Result<impl Stream<Item = Result<Vec<u8>, std::io::Error>> + Send> {
        let (tx, rx) = mpsc::unbounded_channel();

        self.backend_tx
            .send((operation, tx))
            .await
            .map_err(|_| Error::BackendShutDown)?;

        Ok(futures::stream::unfold(rx, |mut rx| async move {
            let res = rx.recv().await?;

            // Some((res.map(), rx))

            match res {
                Ok(data) => Some((Ok(data), rx)),
                Err(err) => Some((Err(std::io::Error::new(std::io::ErrorKind::Other, err)), rx)),
            }
        });

        Ok(raw_data_stream)
    }
}

struct BackGroundWorker<S> {
    websocket: WebSocketStream<S>,
    operation_rx: mpsc::Receiver<OperationMsg>,
    subscriptions: Vec<Option<mpsc::UnboundedSender<WsMsg>>>,
    next_id: u8,
}

impl<S> BackGroundWorker<S>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    fn new(websocket: WebSocketStream<S>, operation_rx: mpsc::Receiver<OperationMsg>) -> Self {
        Self {
            websocket,
            operation_rx,
            subscriptions: vec![None; 256],
            next_id: 0,
        }
    }

    async fn run(mut self) -> Result<()> {
        use futures::future::Either;

        loop {
            let next_ws_msg = self.websocket.next();
            let next_operation = self.operation_rx.recv();
            let ping = tokio::time::sleep(std::time::Duration::from_secs(1));

            let either = {
                futures::pin_mut!(next_operation);

                tokio::select! {
                    val = next_ws_msg => Either::Left(val),
                    val = next_operation => Either::Right(val),
                    _ = ping => {
                        self.websocket.send(Message::Ping(Vec::new())).await?;
                        continue;
                    }
                }
            };

            match either {
                Either::Left(Some(msg)) => self.handle_msg(msg?).await?,
                Either::Right(Some((operation, sender))) => {
                    self.send_request(operation, sender).await?
                }
                Either::Left(None) | Either::Right(None) => break,
            }
        }

        Ok(())
    }

    async fn handle_msg(&mut self, msg: Message) -> Result<()> {
        let data = match msg {
            Message::Binary(data) => data,
            Message::Ping(data) => return self.send_msg(Message::Pong(data)).await,
            Message::Pong(_) => return Ok(()),
            Message::Close(_) => return Err(Error::ConnectionClosed),
            _ => return Err(Error::UnexpectedMessage),
        };

        let (header, data) = Header::try_from_data(data)?;

        let msg = if header.marker.contains(MsgMarker::END) {
            let _ = self.subscriptions[header.id as usize].take();
            return Ok(());
        } else if header.marker.contains(MsgMarker::START) {
            return Ok(());
        } else if header.marker.contains(MsgMarker::ERROR) {
            match String::from_utf8(data) {
                Ok(s) => Err(Error::ErrorMsg(s)),
                Err(_) => Err(Error::UnexpectedMessageFormat),
            }
        } else if header.marker.contains(MsgMarker::CONTINUE) {
            Ok(data)
        } else {
            Err(Error::UnexpectedMessageFormat)
        };

        // Even when the receiver is closed, we have to keep the subscription until the server
        // sends `END`. Otherwise we might reuse the id and get confusing responses.
        // We don't support unsubscribing for WebSocket yet :(
        let _ = self.subscriptions[header.id as usize]
            .as_ref()
            .ok_or(Error::UnknownResponseId)?
            .send(msg);

        Ok(())
    }

    async fn send_request(
        &mut self,
        operation: Operation,
        sender: mpsc::UnboundedSender<WsMsg>,
    ) -> Result<()> {
        let id = self.allocate_id()?;
        let request = Request { id, operation };
        let payload = serde_cbor::to_vec(&request)?;

        self.subscriptions[id as usize] = Some(sender);
        if let Err(err) = self.send_msg(Message::Binary(payload)).await {
            let _ = self.subscriptions[id as usize].take();
            return Err(err);
        }

        Ok(())
    }

    async fn send_msg(&mut self, msg: Message) -> Result<()> {
        self.websocket.send(msg).await?;
        Ok(())
    }

    fn allocate_id(&mut self) -> Result<u8> {
        let id = match self.subscriptions[self.next_id as usize] {
            None => self.next_id,
            Some(_) => self
                .subscriptions
                .iter()
                .enumerate()
                .find(|(_, opt)| opt.is_none())
                .map(|(i, _)| i as u8)
                .ok_or(Error::MaxConcurrentRequestLimitReached)?,
        };

        self.next_id = self.next_id.wrapping_add(1);
        Ok(id)
    }
}

#[derive(serde::Serialize)]
struct Request {
    id: u8,
    #[serde(flatten)]
    operation: Operation,
}

#[derive(Clone, serde::Serialize)]
#[serde(tag = "operation", rename_all = "camelCase")]
pub enum Operation {
    GetPairs {
        pairs: Vec<[u8; 20]>,
        start: Option<u64>,
        end: Option<u64>,
    },
    GetPrices {
        pairs: Vec<[u8; 20]>,
        start: Option<u64>,
        end: Option<u64>,
    },
    GetReserves {
        pairs: Vec<[u8; 20]>,
        start: Option<u64>,
        end: Option<u64>,
    },
    GetHeight,
    GetEvmEvents,
}

struct Header {
    marker: MsgMarker,
    id: u8,
    _counter: u32,
}

impl Header {
    const SIZE: usize = 6;

    fn try_from_data(mut data: Vec<u8>) -> Result<(Self, Vec<u8>)> {
        let data_len = data.len();
        if data_len < Self::SIZE {
            return Err(Error::UnexpectedMessageFormat);
        }

        let header = &data[(data_len - Self::SIZE)..];

        let marker = MsgMarker::from_bits(header[0]).ok_or(Error::UnexpectedMessageFormat)?;
        let id = header[1];
        let _counter = u32::from_be_bytes(header[2..].try_into().unwrap());

        let header = Self {
            marker,
            id,
            _counter,
        };
        data.truncate(data_len - Self::SIZE);

        Ok((header, data))
    }
}

bitflags::bitflags! {
    struct MsgMarker: u8 {
        const START        = 0b00000001;
        const CONTINUE     = 0b00000010;
        const END          = 0b00000100;
        const ERROR        = 0b10000000;
        const SUBSCRIPTION = 0b01000000;
    }
}
