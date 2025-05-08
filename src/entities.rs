use std::fmt::Debug;

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct Protocol {
    name: String,
    types: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Device {
    name: String,
    protocols: Vec<Protocol>,
}

#[derive(Debug, Deserialize)]
pub struct DeviceList {
    devices: Vec<Device>,
}

#[derive(Debug, Deserialize)]
pub struct PlanSpec {
    pub name: String,
    pub description: Option<String>,
    pub schema: Value,
}

#[derive(Debug, Deserialize)]
pub struct PlanList {
    pub plans: Vec<PlanSpec>,
}

impl Debug for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.types.is_empty() {
            f.write_str("[")?;
            f.write_str(&self.types.join(", "))?;
            f.write_str("]")?;
        }
        Ok(())
    }
}
