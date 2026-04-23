//! Socket collection boundary for platform-specific implementations.

use std::{collections::BTreeSet, net::IpAddr};

use netstat2::{AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState, get_sockets_info};

use crate::{core::Protocol, error::Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundSocket {
    pub protocol: Protocol,
    pub bind_addr: IpAddr,
    pub port: u16,
    pub pids: Vec<u32>,
}

pub fn listening_sockets() -> Result<Vec<BoundSocket>> {
    let sockets = get_sockets_info(
        AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6,
        ProtocolFlags::TCP | ProtocolFlags::UDP,
    )?;

    let mut listeners = Vec::new();

    for socket in sockets {
        let Some((protocol, bind_addr, port)) = socket_endpoint(&socket.protocol_socket_info)
        else {
            continue;
        };

        if port == 0 {
            continue;
        }

        listeners.push(BoundSocket {
            protocol,
            bind_addr,
            port,
            pids: dedupe_pids(socket.associated_pids),
        });
    }

    listeners.sort_by(|left, right| {
        left.port
            .cmp(&right.port)
            .then_with(|| left.bind_addr.cmp(&right.bind_addr))
            .then_with(|| left.protocol.cmp(&right.protocol))
            .then_with(|| left.pids.cmp(&right.pids))
    });
    listeners.dedup();

    Ok(listeners)
}

fn socket_endpoint(socket: &ProtocolSocketInfo) -> Option<(Protocol, IpAddr, u16)> {
    match socket {
        ProtocolSocketInfo::Tcp(tcp) if tcp.state == TcpState::Listen => {
            Some((Protocol::Tcp, tcp.local_addr, tcp.local_port))
        }
        ProtocolSocketInfo::Tcp(_) => None,
        ProtocolSocketInfo::Udp(udp) => Some((Protocol::Udp, udp.local_addr, udp.local_port)),
    }
}

fn dedupe_pids(pids: Vec<u32>) -> Vec<u32> {
    pids.into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
