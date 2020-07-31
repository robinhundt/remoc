//! # Channel multiplexer
//!
//! Multiplexes multiple channels over a single channel (or anything that implements Sink and Stream).
//!

mod channel;
mod client;
mod multiplexer;
mod number_allocator;
mod receive_buffer;
mod receiver;
mod send_lock;
mod sender;
mod server;
mod timeout;

pub mod codec;

pub use channel::Channel;
pub use client::{Client, ConnectError};
pub use codec::{ContentCodecFactory, Deserializer, Serializer, TransportCodecFactory};
pub use multiplexer::{Cfg, MultiplexError, MultiplexMsg, Multiplexer};
pub use receiver::{ReceiveError, Receiver};
pub use sender::{HangupNotify, SendError, Sender};
pub use server::{RemoteConnectToServiceRequest, Server, ServerError};

/// Return if connection terminated.
///
/// Argument must be of type `Result<_, SendError>` or `Result<_, ReceiveError>`.
///
/// Returns from the current function (with no return value) if the
/// argument is an error result due to a closed channel or terminated
/// multiplexer. Otherwise the result is passed unmodified.
#[macro_export]
macro_rules! term {
    ($exp:expr) => {
        match $exp {
            Ok(v) => Ok(v),
            Err(err) if err.is_terminated() => return,
            Err(err) => Err(err),
        }
    };
}
