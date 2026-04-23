use serde::Serialize;

use crate::error::Result;

pub fn print<T>(value: &T) -> Result<()>
where
    T: Serialize + ?Sized,
{
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
