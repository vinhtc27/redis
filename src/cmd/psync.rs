use crate::{config::Config, Connection, Frame, Parse};
use bytes::Bytes;
use tracing::{debug, instrument};

use base64::{engine::general_purpose, Engine};
const EMPTY_RDB_FILE_BASE64: &[u8; 120] = b"UkVESVMwMDEx+glyZWRpcy12ZXIFNy4yLjD6CnJlZGlzLWJpdHPAQPoFY3RpbWXCbQi8ZfoIdXNlZC1tZW3CsMQQAPoIYW9mLWJhc2XAAP/wbjv+wP9aog==";

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
    pub(crate) async fn apply(
        self,
        config: &mut Config,
        dst: &mut Connection,
    ) -> crate::Result<()> {
        if self.replicationid == "?" && self.offset == -1 {
            config.set_second_repl_offset(0);
        };
        let (replicationid, offset) = config.master_replid_and_offset();

        let fullresync_response = Frame::Simple(format!("FULLRESYNC {} {}", replicationid, offset));
        debug!(?fullresync_response);
        dst.write_frame(&fullresync_response).await?;

        let empty_rdb_file = general_purpose::STANDARD.decode(EMPTY_RDB_FILE_BASE64)?;

        let rdb_response = Frame::File(empty_rdb_file.into());
        debug!(?rdb_response);
        dst.write_frame(&rdb_response).await?;

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
