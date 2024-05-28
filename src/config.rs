use std::fmt::{Display, Formatter, Result};

use crate::{util, DEFAULT_PORT};

#[derive(Debug, Clone)]
pub struct Config {
    server_config: ServerConfig,
    replication_config: ReplicationConfig,
}

impl Config {
    pub fn new(port: Option<u16>, is_replication: bool) -> Self {
        Config {
            server_config: ServerConfig {
                tcp_port: match port {
                    Some(port) => port,
                    None => DEFAULT_PORT,
                },
            },
            replication_config: ReplicationConfig {
                role: if is_replication {
                    ReplicationRole::Slave
                } else {
                    ReplicationRole::Master
                },
                connected_slaves: 0,
                master_replid: if is_replication {
                    String::new()
                } else {
                    util::random_string(40)
                },
                master_repl_offset: 0,
                second_repl_offset: 0,
                repl_backlog_active: 0,
                repl_backlog_size: 1048576,
                repl_backlog_first_byte_offset: 0,
                repl_backlog_histlen: 0,
            },
        }
    }

    pub fn role(&self) -> &ReplicationRole {
        &self.replication_config.role
    }

    pub fn server(&self) -> String {
        self.server_config.to_string()
    }

    pub fn server_tcp_port(&self) -> u16 {
        self.server_config.tcp_port
    }

    pub fn replication(&self) -> String {
        self.replication_config.to_string()
    }

    pub fn master_replid_and_offset(&self) -> (String, usize) {
        (
            self.replication_config.master_replid.clone(),
            self.replication_config.master_repl_offset,
        )
    }
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    tcp_port: u16,
}

impl Display for ServerConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "# Server\n\
             tcp_port:{}\n",
            self.tcp_port,
        )
    }
}

#[derive(Debug, Clone)]
pub struct ReplicationConfig {
    role: ReplicationRole,
    connected_slaves: usize,
    master_replid: String,
    master_repl_offset: usize,
    second_repl_offset: usize,
    repl_backlog_active: usize,
    repl_backlog_size: usize,
    repl_backlog_first_byte_offset: usize,
    repl_backlog_histlen: usize,
}

impl Display for ReplicationConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "# Replication\n\
             role:{}\n\
             connected_slaves:{}\n\
             master_replid:{}\n\
             master_repl_offset:{}\n\
             second_repl_offset:{}\n\
             repl_backlog_active:{}\n\
             repl_backlog_size:{}\n\
             repl_backlog_first_byte_offset:{}\n\
             repl_backlog_histlen:{}\n",
            self.role,
            self.connected_slaves,
            self.master_replid,
            self.master_repl_offset,
            self.second_repl_offset,
            self.repl_backlog_active,
            self.repl_backlog_size,
            self.repl_backlog_first_byte_offset,
            self.repl_backlog_histlen
        )
    }
}

#[derive(Debug, Clone)]
pub enum ReplicationRole {
    Master,
    Slave,
}

impl From<&str> for ReplicationRole {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "master" => ReplicationRole::Master,
            "slave" => ReplicationRole::Slave,
            _ => panic!("Invalid replication role: '{}'", s),
        }
    }
}

impl Display for ReplicationRole {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ReplicationRole::Master => write!(f, "master"),
            ReplicationRole::Slave => write!(f, "slave"),
        }
    }
}
