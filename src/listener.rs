use std::net::SocketAddr;

use sysinfo::Pid;

use listeners::{Protocol, SocketState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListeningSocket {
    pub(crate) pid: Pid,
    pub(crate) socket: SocketAddr,
    protocol: Protocol,
}

pub fn get_listening_sockets() -> Result<Vec<ListeningSocket>, String> {
    let mut ports = Vec::new();

    match listeners::get_all() {
        Ok(listeners) => {
            for l in listeners {
                if let SocketState::Listen = l.state
                    && l.protocol == Protocol::TCP
                {
                    ports.push(ListeningSocket {
                        pid: Pid::from_u32(l.process.pid),
                        socket: l.socket,
                        protocol: l.protocol,
                    })
                }
            }

            Ok(ports)
        }
        Err(e) => Err(e.to_string()),
    }
}
