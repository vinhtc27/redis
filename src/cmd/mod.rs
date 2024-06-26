mod ping;
use std::sync::atomic::AtomicUsize;

pub use ping::Ping;

mod echo;
pub use echo::Echo;

mod info;
pub use info::Info;

mod replconf;
pub use replconf::ReplConf;

mod psync;
pub use psync::PSync;

mod get;
pub use get::Get;

mod set;
pub use set::Set;

mod publish;
pub use publish::Publish;

mod subscribe;
pub use subscribe::{Subscribe, Unsubscribe};

mod unknown;
pub use unknown::Unknown;

use crate::{config::Config, Connection, Db, Frame, Parse, ParseError, Shutdown};

/// Enumeration of supported Redis commands.
///
/// Methods called on `Command` are delegated to the command implementation.
#[derive(Debug)]
pub enum Command {
    Ping(Ping),
    Echo(Echo),
    Info(Info),
    ReplConf(ReplConf),
    PSync(PSync),
    Get(Get),
    Set(Set),
    Publish(Publish),
    Subscribe(Subscribe),
    Unsubscribe(Unsubscribe),
    Unknown(Unknown),
}

impl Command {
    /// Parse a command from a received frame.
    ///
    /// The `Frame` must represent a Redis command supported by `redis` and
    /// be the array variant.
    ///
    /// # Returns
    ///
    /// On success, the command value is returned, otherwise, `Err` is returned.
    pub fn from_frame(frame: Frame) -> crate::Result<Command> {
        // The frame value is decorated with `Parse`. `Parse` provides a
        // "cursor" like API which makes parsing the command easier.
        //
        // The frame value must be an array variant. Any other frame variants
        // result in an error being returned.
        let mut parse = Parse::new(frame)?;

        // All redis commands begin with the command name as a string. The name
        // is read and converted to lower cases in order to do case sensitive
        // matching.
        let command_name = parse.next_string()?.to_lowercase();

        // Match the command name, delegating the rest of the parsing to the
        // specific command.
        let command = match &command_name[..] {
            "ping" => Command::Ping(Ping::parse_frames(&mut parse)?),
            "echo" => Command::Echo(Echo::parse_frames(&mut parse)?),
            "info" => Command::Info(Info::parse_frames(&mut parse)?),
            "replconf" => Command::ReplConf(ReplConf::parse_frames(&mut parse)?),
            "psync" => Command::PSync(PSync::parse_frames(&mut parse)?),
            "get" => Command::Get(Get::parse_frames(&mut parse)?),
            "set" => Command::Set(Set::parse_frames(&mut parse)?),
            "publish" => Command::Publish(Publish::parse_frames(&mut parse)?),
            "subscribe" => Command::Subscribe(Subscribe::parse_frames(&mut parse)?),
            "unsubscribe" => Command::Unsubscribe(Unsubscribe::parse_frames(&mut parse)?),
            _ => {
                // The command is not recognized and an Unknown command is
                // returned.
                //
                // `return` is called here to skip the `finish()` call below. As
                // the command is not recognized, there is most likely
                // unconsumed fields remaining in the `Parse` instance.
                return Ok(Command::Unknown(Unknown::new(command_name)));
            }
        };

        // Check if there is any remaining unconsumed fields in the `Parse`
        // value. If fields remain, this indicates an unexpected frame format
        // and an error is returned.
        parse.finish()?;

        // The command has been successfully parsed
        Ok(command)
    }

    /// Apply the command to the specified `Db` instance.
    ///
    /// The response is written to `dst`. This is called by the server in order
    /// to execute a received command.
    pub(crate) async fn apply(
        self,
        db: &Db,
        config: &mut Config,
        dst: &mut Connection,
        shutdown: &mut Shutdown,
        offset: Option<&AtomicUsize>,
    ) -> crate::Result<()> {
        use Command::*;

        match self {
            Ping(cmd) => cmd.apply(config, dst).await?,
            Echo(cmd) => cmd.apply(config, dst).await?,
            Info(cmd) => cmd.apply(config, dst).await?,
            ReplConf(cmd) => cmd.apply(config, db, dst, offset).await?,
            PSync(cmd) => cmd.apply(config, dst).await?,
            Get(cmd) => cmd.apply(config, db, dst).await?,
            Set(cmd) => cmd.apply(config, db, dst).await?,
            Publish(cmd) => cmd.apply(config, db, dst).await?,
            Subscribe(cmd) => cmd.apply(config, db, dst, shutdown).await?,
            // `Unsubscribe` cannot be applied. It may only be received from the
            // context of a `Subscribe` command.
            Unsubscribe(_) => return Err("`Unsubscribe` is unsupported in this context".into()),
            Unknown(cmd) => cmd.apply(config, dst).await?,
        }

        Ok(())
    }

    /// Returns the command name
    pub(crate) fn get_name(&self) -> &str {
        match self {
            Command::Ping(_) => "ping",
            Command::Echo(_) => "echo",
            Command::Info(_) => "info",
            Command::ReplConf(_) => "replconf",
            Command::PSync(_) => "psync",
            Command::Get(_) => "get",
            Command::Set(_) => "set",
            Command::Publish(_) => "pub",
            Command::Subscribe(_) => "subscribe",
            Command::Unsubscribe(_) => "unsubscribe",
            Command::Unknown(cmd) => cmd.get_name(),
        }
    }
}
