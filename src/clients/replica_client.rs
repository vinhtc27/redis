use crate::cmd::Ping;
use crate::{Connection, Frame};

use bytes::Bytes;
use std::io::{Error, ErrorKind};
use tokio::net::{TcpStream, ToSocketAddrs};
use tracing::{debug, instrument};

#[derive(Debug)]
pub struct ReplicaClient {
    connection: Connection,
}

impl ReplicaClient {
    pub async fn connect<T: ToSocketAddrs>(
        addr: Option<T>,
    ) -> crate::Result<Option<ReplicaClient>> {
        if addr.is_none() {
            return Ok(None);
        }
        let socket = TcpStream::connect(addr.unwrap()).await?;
        let connection = Connection::new(socket);

        Ok(Some(ReplicaClient { connection }))
    }

    #[instrument(skip(self))]
    pub async fn ping(&mut self, msg: Option<Bytes>) -> crate::Result<Bytes> {
        let frame = Ping::new(msg).into_frame();
        debug!(request = ?frame);
        self.connection.write_frame(&frame).await?;

        match self.read_response().await? {
            Frame::Simple(value) => Ok(value.into()),
            Frame::Bulk(value) => Ok(value),
            frame => Err(frame.to_error()),
        }
    }

    async fn read_response(&mut self) -> crate::Result<Frame> {
        let response = self.connection.read_frame().await?;

        debug!(?response);

        match response {
            Some(Frame::Error(msg)) => Err(msg.into()),
            Some(frame) => Ok(frame),
            None => {
                let err = Error::new(ErrorKind::ConnectionReset, "connection reset by server");
                Err(err.into())
            }
        }
    }
}
