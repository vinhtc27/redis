use bytes::Bytes;
use tracing::{debug, instrument};

use crate::{
    config::{Config, ReplicationRole},
    parse::{Parse, ParseError},
    Connection, Frame,
};

/// Returns message sended by client
///
/// ECHO is a command like PING
/// that's used for testing and debugging.
#[derive(Debug, Default)]
pub struct Echo {
    msg: Bytes,
}

impl Echo {
    /// Create a new `Echo` command with `msg`.
    pub fn new(msg: Bytes) -> Echo {
        Echo { msg }
    }

    /// Parse a `Echo` instance from a received frame.
    ///
    /// The `Parse` argument provides a cursor-like API to read fields from the
    /// `Frame`. At this point, the entire frame has already been received from
    /// the socket.
    ///
    /// The `ECHO` string has already been consumed.
    ///
    /// # Returns
    ///
    /// Returns the `Echo` value on success. If the frame is malformed, `Err` is
    /// returned.
    ///
    /// # Format
    ///
    /// Expects an array frame containing `ECHO` and an optional message.
    ///
    /// ```text
    /// ECHO [message]
    /// ```
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Echo> {
        match parse.next_bytes() {
            Ok(msg) => Ok(Echo::new(msg)),
            Err(ParseError::EndOfStream) => Ok(Echo::default()),
            Err(e) => Err(e.into()),
        }
    }

    /// Apply the `Echo` command and return the message.
    ///
    /// The response is written to `dst`. This is called by the server in order
    /// to execute a received command.
    #[instrument(skip(self, dst))]
    pub(crate) async fn apply(self, config: &Config, dst: &mut Connection) -> crate::Result<()> {
        let response = Frame::Bulk(self.msg);

        debug!(?response);
        match config.role() {
            ReplicationRole::Master => dst.write_frame(&response).await?,
            ReplicationRole::Slave => {}
        }

        Ok(())
    }

    /// Converts the command into an equivalent `Frame`.
    ///
    /// This is called by the client when encoding a `Echo` command to send
    /// to the server.
    pub(crate) fn into_frame(self) -> Frame {
        let mut frame = Frame::array();
        frame.push_bulk(Bytes::from("echo".as_bytes()));
        frame.push_bulk(self.msg);
        frame
    }
}
