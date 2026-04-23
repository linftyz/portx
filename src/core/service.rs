use crate::{
    core::{ListenerRecord, PortDetails, Scope},
    error::Result,
    platform::sockets,
};

#[derive(Debug, Default)]
pub struct PortService;

impl PortService {
    pub fn list(&self, scope: Option<Scope>) -> Result<Vec<ListenerRecord>> {
        let mut records = sockets::listening_sockets()?
            .into_iter()
            .flat_map(|socket| {
                let scope = Scope::classify(socket.bind_addr);

                if socket.pids.is_empty() {
                    return vec![ListenerRecord {
                        port: socket.port,
                        protocol: socket.protocol,
                        bind_addr: socket.bind_addr,
                        scope,
                        pid: None,
                        process_name: None,
                        command: None,
                    }];
                }

                socket
                    .pids
                    .into_iter()
                    .map(|pid| ListenerRecord {
                        port: socket.port,
                        protocol: socket.protocol,
                        bind_addr: socket.bind_addr,
                        scope,
                        pid: Some(pid),
                        process_name: None,
                        command: None,
                    })
                    .collect()
            })
            .filter(|record| scope.is_none_or(|scope| record.scope == scope))
            .collect::<Vec<_>>();

        sort_listener_records(&mut records);
        Ok(records)
    }

    pub fn info(&self, _port: u16, _pid: Option<u32>) -> Result<Vec<PortDetails>> {
        Ok(Vec::new())
    }

    pub fn find(&self, _process_name: &str, _scope: Option<Scope>) -> Result<Vec<ListenerRecord>> {
        Ok(Vec::new())
    }

    pub fn kill(&self, _port: u16, _pid: Option<u32>, _force: bool, _yes: bool) -> Result<()> {
        Ok(())
    }

    pub fn watch(&self, _port: u16, _pid: Option<u32>) -> Result<()> {
        Ok(())
    }
}

fn sort_listener_records(records: &mut [ListenerRecord]) {
    records.sort_by(|left, right| {
        left.port
            .cmp(&right.port)
            .then_with(|| left.bind_addr.cmp(&right.bind_addr))
            .then_with(|| protocol_rank(left.protocol).cmp(&protocol_rank(right.protocol)))
            .then_with(|| left.pid.cmp(&right.pid))
    });
}

fn protocol_rank(protocol: crate::core::Protocol) -> u8 {
    match protocol {
        crate::core::Protocol::Tcp => 0,
        crate::core::Protocol::Udp => 1,
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use crate::core::Protocol;

    use super::*;

    #[test]
    fn sorts_listener_records_by_port_address_protocol_and_pid() {
        let mut records = vec![
            record(3000, Ipv4Addr::new(127, 0, 0, 1), Protocol::Udp, Some(2)),
            record(2000, Ipv4Addr::new(127, 0, 0, 1), Protocol::Tcp, Some(1)),
            record(3000, Ipv4Addr::new(0, 0, 0, 0), Protocol::Tcp, None),
            record(3000, Ipv4Addr::new(127, 0, 0, 1), Protocol::Tcp, Some(1)),
        ];

        sort_listener_records(&mut records);

        let ordered = records
            .iter()
            .map(|record| (record.port, record.bind_addr, record.protocol, record.pid))
            .collect::<Vec<_>>();

        assert_eq!(
            ordered,
            vec![
                (
                    2000,
                    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                    Protocol::Tcp,
                    Some(1)
                ),
                (
                    3000,
                    IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                    Protocol::Tcp,
                    None
                ),
                (
                    3000,
                    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                    Protocol::Tcp,
                    Some(1)
                ),
                (
                    3000,
                    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                    Protocol::Udp,
                    Some(2)
                ),
            ]
        );
    }

    fn record(
        port: u16,
        bind_addr: Ipv4Addr,
        protocol: Protocol,
        pid: Option<u32>,
    ) -> ListenerRecord {
        ListenerRecord {
            port,
            protocol,
            bind_addr: IpAddr::V4(bind_addr),
            scope: Scope::classify(IpAddr::V4(bind_addr)),
            pid,
            process_name: None,
            command: None,
        }
    }
}
