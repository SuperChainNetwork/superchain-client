//! SC-gateway
//! A simple API wrapper for the Superchain gateway
//!
//! Superchain is a start up with the mission provide easy and fast access to on chain data.
//! This crate gives you easy access to a hand full of our endpoints. Namely `get_pair` and
//! `get_prices`. These endpoints allow you to get a stream of all pair created events and all
//! price quotes of uniswap v2.
//!
//! ### Introduction
//! This crates allows you to easily use the Superchain API. Both the WebSocket endpoints and the
//! HTTP endpoints.
//!
//! ### API overview
//! There are two ways to interface with Superchain: HTTP and WebSocket
//!
//! The WebSocket interface is a lot more flexible and powerful, while also being simpler, so use
//! this one whenever you can.
//!
//! #### HTTP
//!
//! - [`HttpClient::get_pair_created`]\: Get the PairCreated event for a pair from the entire eth history
//! - [`HttpClient::get_pair_created_in_range`]\: Get the PairCreated event for a pair from the provided block range
//! - [`HttpClient::get_pair_created_live_stream`]\: Get the PairCreated event for a pair from the provided block and keep streaming from head
//! - [`HttpClient::get_prices_in_range`]\: Get all price quotes for a pair from the provided block range
//! - [`HttpClient::get_prices_live_stream`]\: Get all price quotes for a pair from the provided block range and keep streaming from head
//!
//! #### WebSocket
//!
//! - [`WsClient::get_pairs_created`]\: Get the PairCreated event for a pair from the specified block range
//! - [`WsClient::get_prices`]\: Get all price quotes for a pair from the specified block range

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]

pub use ::{ethers, futures, reqwest, tokio, tokio_tungstenite, tungstenite, url};

#[doc(inline)]
pub use crate::{
    error::{Error, Result},
    http::Client as HttpClient,
    types::{PairCreated, Price, QueryOptions, Reserves, Side, Type},
    ws::Client as WsClient,
};

pub mod config;
mod error;
mod http;
mod types;
mod ws;
