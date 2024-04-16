use crate::{Connection, Frame, Parse, ParseError};
use bytes::Bytes;
use tracing::{debug, instrument};

#[derive(Debug, Default)]
pub struct Info {
    sections: Option<Vec<Bytes>>,
}

impl Info {
    pub fn new(sections: Option<Vec<Bytes>>) -> Info {
        Info { sections }
    }

    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Info> {
        let mut sections = vec![];
        loop {
            match parse.next_bytes() {
                Ok(section) => sections.push(section),
                Err(ParseError::EndOfStream) => break,
                Err(e) => return Err(e.into()),
            }
        }
        Ok(Info::new(Some(sections)))
    }

    #[instrument(skip(self, dst))]
    pub(crate) async fn apply(self, dst: &mut Connection) -> crate::Result<()> {
        let response = match self.sections {
            None => unimplemented!(),
            Some(sections) => match sections.len() {
                1 => {
                    if std::str::from_utf8(sections.first().unwrap()).unwrap() == "replication" {
                        let info = "# Replication\nrole:master\nconnected_slaves:0\nmaster_failover_state:no-failover\nmaster_replid:c7282793444f6422eeeeadb2a27619744abba897\nmaster_replid2:0000000000000000000000000000000000000000\nmaster_repl_offset:0\nsecond_repl_offset:-1\nrepl_backlog_active:0\nrepl_backlog_size:1048576\nrepl_backlog_first_byte_offset:0\nrepl_backlog_histlen:0";
                        Frame::Bulk(Bytes::from(info.as_bytes()))
                    } else {
                        unimplemented!()
                    }
                }
                _ => unimplemented!(),
            },
        };

        debug!(?response);

        dst.write_frame(&response).await?;

        Ok(())
    }

    pub(crate) fn into_frame(self) -> Frame {
        let mut frame = Frame::array();
        frame.push_bulk(Bytes::from("info".as_bytes()));
        if let Some(sections) = self.sections {
            for section in sections {
                frame.push_bulk(section);
            }
        }
        frame
    }
}
