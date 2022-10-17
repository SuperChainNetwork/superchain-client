use ethers::types::H160;
use futures::{Stream, StreamExt, TryStreamExt};

use crate::{
    types::{BlockHeader, PairCreated, Price, Reserves},
    Error, QueryOptions, Result,
};

/// A Superchain HTTP client
pub struct Client {
    inner: reqwest::Client,
    headers: reqwest::header::HeaderMap,
    base_url: reqwest::Url,
}

impl Client {
    /// Create a new [`Client`] with the specified API `base_url`
    ///
    /// `base_url` is the URL of the Superchain server without any path suffixes, like
    /// `http://localhost:8097/` or `https://123.4.5.123:8080/`.
    pub fn new(client: reqwest::Client, base_url: reqwest::Url) -> Self {
        Self {
            inner: client,
            headers: reqwest::header::HeaderMap::new(),
            base_url,
        }
    }

    /// Set the default headers provided for each request
    ///
    /// This can be useful if you need to i.e. provide a basic auth header.
    pub fn with_default_headers(mut self, headers: reqwest::header::HeaderMap) -> Self {
        self.headers = headers;
        self
    }

    /// Get the uniswap v2 pair created events for the provided `pair`
    pub async fn get_pair_created(
        &self,
        pair: H160,
        opts: QueryOptions,
    ) -> Result<Option<PairCreated>> {
        let url = self.base_url.join(&format!("/api/eth/pair/{:x}", pair))?;
        self.request(url, opts).await?.next().await.transpose()
    }

    /// Get the uniswap v2 prices for the provided `pair`
    pub async fn get_prices(
        &self,
        pair: H160,
        opts: QueryOptions,
    ) -> Result<impl Stream<Item = Result<Price>> + Send> {
        let url = self.base_url.join(&format!("/api/eth/prices/{:x}", pair))?;
        self.request(url, opts).await
    }

    /// Get the uniswap v2 reserves for the provided `pair`
    pub async fn get_reserves(
        &self,
        pair: H160,
        opts: QueryOptions,
    ) -> Result<impl Stream<Item = Result<Reserves>> + Send> {
        let url = self
            .base_url
            .join(&format!("/api/eth/reserves/{:x}", pair))?;
        self.request(url, opts).await
    }

    /// Get the block headers
    pub async fn get_headers(
        &self,
        opts: QueryOptions,
    ) -> Result<impl Stream<Item = Result<BlockHeader>> + Send> {
        let url = self.base_url.join("/api/eth/headers/")?;
        self.request(url, opts).await
    }

    pub async fn get_height(&self) -> Result<u64> {
        let height = self
            .inner
            .get("/api/eth/height")
            .send()
            .await?
            .error_for_status()?
            .json::<u64>()
            .await?;
        Ok(height)
    }

    async fn request<T>(
        &self,
        url: url::Url,
        opts: QueryOptions,
    ) -> Result<impl Stream<Item = Result<T>> + Send>
    where
        T: serde::de::DeserializeOwned + 'static,
    {
        let raw_data_stream = self
            .inner
            .get(url)
            .query(&opts)
            .headers(self.headers.clone())
            .send()
            .await?
            .error_for_status()?
            .bytes_stream()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err));

        let stream = csv_async::AsyncDeserializer::from_reader(raw_data_stream.into_async_read())
            .into_deserialize()
            .map_err(Error::from)
            .into_stream();
        Ok(stream)
    }
}
