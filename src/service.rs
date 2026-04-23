use crate::{
    domain::{ListenerRecord, PortDetails, Scope},
    error::Result,
};

#[derive(Debug, Default)]
pub struct PortService;

impl PortService {
    pub fn list(&self, _scope: Option<Scope>) -> Result<Vec<ListenerRecord>> {
        Ok(Vec::new())
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
