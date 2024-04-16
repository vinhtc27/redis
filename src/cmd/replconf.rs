use crate::{Connection, Frame, Parse};
use bytes::Bytes;
use tracing::{debug, instrument};

#[derive(Debug, Default)]
pub struct ReplConf {
    #[allow(dead_code)]
    key: Option<String>,
    #[allow(dead_code)]
    value: Option<Bytes>,
}

impl ReplConf {
    pub fn new(key: impl ToString, value: Bytes) -> ReplConf {
        ReplConf {
            key: Some(key.to_string()),
            value: Some(value),
        }
    }

    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<ReplConf> {
        let key = parse.next_string()?;
        let value = parse.next_bytes()?;

        Ok(ReplConf::new(key, value))
    }

    #[instrument(skip(self, dst))]
    pub(crate) async fn apply(self, dst: &mut Connection) -> crate::Result<()> {
        let response = Frame::Simple("OK".to_string());
        debug!(?response);

        dst.write_frame(&response).await?;

        Ok(())
    }

    pub(crate) fn into_frame(self) -> Frame {
        let mut frame = Frame::array();
        frame.push_bulk(Bytes::from("replconf".as_bytes()));
        if self.key.is_some() {
            frame.push_bulk(Bytes::from(self.key.unwrap().into_bytes()));
        }
        if self.value.is_some() {
            frame.push_bulk(self.value.unwrap());
        }
        frame
    }
}
