//! Error handling for Rust
//!
//! This module defines various utilities for handling errors in the Rust
//! thread. This uses the `error-chain` crate to ergonomically define errors,
//! enable them for usage with `?`, and otherwise give us some nice utilities.
//! It's expected that this module is always glob imported:
//!
//! ```ignore
//!     use errors::*;
//! ```
//!
//! And functions in general should then return `Result<()>`. You can add extra
//! error context via `chain_err`:
//!
//! ```ignore
//!     let e = some_function_returning_a_result().chain_err(|| {
//!         "some extra context here to make a nicer error"
//!     })?;
//! ```
//!
//! And you can also use the `MyFuture` type alias for "nice" uses of futures
//!
//! ```ignore
//!     fn add(a: i32) -> MyFuture<u32> {
//!         // ..
//!     }
//! ```
//!
//! You can find some more documentation about this in the `error-chain` crate
//! online.

use std::any::Any;
use std::error;
use std::io;
use std::num;

use cadence;
use config;
use futures::Future;
use httparse;
use sentry;
use serde_json;
use tungstenite;
use uuid;

error_chain! {
    foreign_links {
        Ws(tungstenite::Error);
        Io(io::Error);
        Json(serde_json::Error);
        Httparse(httparse::Error);
        MetricError(cadence::MetricError);
        SentryError(sentry::Error);
        UuidParseError(uuid::ParseError);
        ParseIntError(num::ParseIntError);
        ConfigError(config::ConfigError);
    }

    errors {
        Thread(payload: Box<Any + Send>) {
            description("thread panicked")
        }
    }
}

pub type MyFuture<T> = Box<Future<Item = T, Error = Error>>;

pub trait FutureChainErr<T> {
    fn chain_err<F, E>(self, callback: F) -> MyFuture<T>
    where
        F: FnOnce() -> E + 'static,
        E: Into<ErrorKind>;
}

impl<F> FutureChainErr<F::Item> for F
where
    F: Future + 'static,
    F::Error: error::Error + Send + 'static,
{
    fn chain_err<C, E>(self, callback: C) -> MyFuture<F::Item>
    where
        C: FnOnce() -> E + 'static,
        E: Into<ErrorKind>,
    {
        Box::new(self.then(|r| r.chain_err(callback)))
    }
}
