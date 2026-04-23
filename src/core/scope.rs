use std::fmt;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    Public,
    Lan,
    Local,
}

impl fmt::Display for Scope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Public => formatter.write_str("PUBLIC"),
            Self::Lan => formatter.write_str("LAN"),
            Self::Local => formatter.write_str("LOCAL"),
        }
    }
}
