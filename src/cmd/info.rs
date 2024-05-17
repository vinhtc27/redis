use std::str::from_utf8;

use crate::{config::Config, Connection, Frame, Parse, ParseError};
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
    pub(crate) async fn apply(self, config: &Config, dst: &mut Connection) -> crate::Result<()> {
        let response = match self.sections {
            None => unimplemented!(),
            Some(sections) => match sections.len() {
                1 => {
                    let section = from_utf8(sections.first().unwrap()).unwrap().to_lowercase();
                    if section == "replication" {
                        Frame::Bulk(Bytes::from(config.replication()))
                    } else if section == "server" {
                        Frame::Bulk(Bytes::from(config.server()))
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
