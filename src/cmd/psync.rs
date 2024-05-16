use crate::{config::Config, Connection, Frame, Parse};

use bytes::Bytes;
use tracing::{debug, instrument};

static EMPTY_RDB_FILE: [u8; 88] = [
    0x52, 0x45, 0x44, 0x49, 0x53, 0x30, 0x30, 0x31, 0x31, 0xfa, 0x09, 0x72, 0x65, 0x64, 0x69, 0x73,
    0x2d, 0x76, 0x65, 0x72, 0x05, 0x37, 0x2e, 0x32, 0x2e, 0x30, 0xfa, 0x0a, 0x72, 0x65, 0x64, 0x69,
    0x73, 0x2d, 0x62, 0x69, 0x74, 0x73, 0xc0, 0x40, 0xfa, 0x05, 0x63, 0x74, 0x69, 0x6d, 0x65, 0xc2,
    0x6d, 0x08, 0xbc, 0x65, 0xfa, 0x08, 0x75, 0x73, 0x65, 0x64, 0x2d, 0x6d, 0x65, 0x6d, 0xc2, 0xb0,
    0xc4, 0x10, 0x00, 0xfa, 0x08, 0x61, 0x6f, 0x66, 0x2d, 0x62, 0x61, 0x73, 0x65, 0xc0, 0x00, 0xff,
    0xf0, 0x6e, 0x3b, 0xfe, 0xc0, 0xff, 0x5a, 0xa2,
];

#[derive(Debug, Default)]
pub struct PSync {
    replicationid: String,
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

        let rdb_response = Frame::File(EMPTY_RDB_FILE.as_slice().into());
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
