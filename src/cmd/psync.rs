use crate::{util, Connection, Frame, Parse};
use bytes::Bytes;
use tracing::{debug, instrument};

#[derive(Debug, Default)]
pub struct PSync {
    #[allow(dead_code)]
    replicationid: String,
    #[allow(dead_code)]
    offset: i64,
}

impl PSync {
    pub fn new(replicationid: impl ToString, offset: i64) -> PSync {
        PSync {
            replicationid: replicationid.to_string(),
            offset,
        }
    }

    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<PSync> {
        let replicationid = parse.next_string()?;
        let offset = parse.next_int()?;

        Ok(PSync::new(replicationid, offset))
    }

    #[instrument(skip(self, dst))]
    pub(crate) async fn apply(self, dst: &mut Connection) -> crate::Result<()> {
        let (replicationid, offset) = if self.replicationid == "?" && self.offset == -1 {
            (util::random_string(40), 0.to_string())
        } else {
            (self.replicationid.to_owned(), self.offset.to_string())
        };
        let response = Frame::Simple(format!("FULLRESYNC {} {}", replicationid, offset));
        debug!(?response);

        dst.write_frame(&response).await?;

        Ok(())
    }

    pub(crate) fn into_frame(self) -> Frame {
        let mut frame = Frame::array();
        frame.push_bulk(Bytes::from("psync".as_bytes()));
        frame.push_bulk(Bytes::from(self.replicationid));
        frame.push_bulk(Bytes::from(self.offset.to_string()));
        frame
    }
}
