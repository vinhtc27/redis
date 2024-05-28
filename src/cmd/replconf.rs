use crate::{
    config::{Config, ReplicationRole},
    db::Db,
    parse::ParseError,
    Connection, Frame, Parse,
};

use bytes::Bytes;
use std::{
    str::from_utf8,
    sync::atomic::{AtomicUsize, Ordering},
};
use tracing::{debug, instrument};

#[derive(Debug, Default)]
pub struct ReplConf {
    pairs: Option<Vec<(String, Bytes)>>,
}

impl ReplConf {
    pub fn new(pairs: Vec<(impl ToString, Bytes)>) -> ReplConf {
        ReplConf {
            pairs: Some(
                pairs
                    .into_iter()
                    .map(|(key, value)| (key.to_string(), value))
                    .collect(),
            ),
        }
    }

    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<ReplConf> {
        let mut pairs = vec![];
        loop {
            let key = match parse.next_string() {
                Ok(key) => key,
                Err(ParseError::EndOfStream) => break,
                Err(e) => return Err(e.into()),
            };
            let value = match parse.next_bytes() {
                Ok(value) => value,
                Err(ParseError::EndOfStream) => break,
                Err(e) => return Err(e.into()),
            };
            pairs.push((key, value));
        }

        Ok(ReplConf::new(pairs))
    }

    #[instrument(skip(self, dst))]
    pub(crate) async fn apply(
        self,
        config: &Config,
        db: &Db,
        dst: &mut Connection,
        offset: Option<&AtomicUsize>,
    ) -> crate::Result<()> {
        let mut response = Frame::Null;

        let offset_bytes = if offset.is_some() {
            offset.unwrap().load(Ordering::SeqCst).to_string()
        } else {
            "0".to_owned()
        };

        if self.pairs.is_some() {
            match config.role() {
                ReplicationRole::Slave => {
                    let pairs = self.pairs.unwrap();
                    if pairs.len() == 1
                        && pairs[0].0.to_ascii_uppercase() == "GETACK"
                        && from_utf8(&pairs[0].1)? == "*"
                    {
                        response = Frame::Array(vec![
                            Frame::Bulk(Bytes::from("REPLCONF".as_bytes())),
                            Frame::Bulk(Bytes::from("ACK".as_bytes())),
                            Frame::Bulk(Bytes::from(offset_bytes)),
                        ]);
                    }
                }
                ReplicationRole::Master => {}
            }
        }

        if let Frame::Null = response {
            response = Frame::Simple("OK".to_string());
        }

        debug!(?response);
        dst.write_frame(&response).await?;

        Ok(())
    }

    pub(crate) fn into_frame(self) -> Frame {
        let mut frame = Frame::array();
        frame.push_bulk(Bytes::from("replconf".as_bytes()));
        if self.pairs.is_some() {
            for pair in self.pairs.unwrap() {
                frame.push_bulk(Bytes::from(pair.0.into_bytes()));
                frame.push_bulk(pair.1);
            }
        }

        frame
    }
}
