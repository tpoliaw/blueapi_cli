use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One of the bluesky protocols than can be implemented by devices in blueapi
#[derive(Deserialize)]
pub struct Protocol {
    name: String,
    types: Vec<String>,
}

/// Device available in blueapi along with the protocols it implements
#[derive(Debug, Deserialize)]
pub struct Device {
    name: String,
    protocols: Vec<Protocol>,
}

/// List of devices as returned by the blueapi server
#[derive(Debug, Deserialize)]
pub struct DeviceList {
    devices: Vec<Device>,
}

/// Details of a plan available in blueapi
#[derive(Debug, Deserialize)]
pub struct PlanSpec {
    pub name: String,
    pub description: Option<String>,
    pub schema: Value,
}

/// List of plans as returned by the blueapi server
#[derive(Debug, Deserialize)]
pub struct PlanList {
    pub plans: Vec<PlanSpec>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskId(pub String);

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskReference {
    pub task_id: TaskId,
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
