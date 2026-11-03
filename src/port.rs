use std::net::SocketAddr;
use sysinfo::Pid;

use listeners::{Protocol, SocketState};

#[derive(Debug)]
pub struct Port {
    pid: Pid,
    socket: SocketAddr,
    protocol: Protocol,
}

pub fn get_ports() -> Vec<Port> {
    let mut ports = Vec::new();

    if let Ok(listeners) = listeners::get_all() {
        for l in listeners {
            if let SocketState::Listen = l.state {
                ports.push(Port {
                    pid: Pid::from_u32(l.process.pid),
                    socket: l.socket,
                    protocol: l.protocol,
                })
            }
        }
    }

    ports
}
