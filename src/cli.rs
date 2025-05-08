use clap::Parser;
use serde_json::Value;

#[derive(Debug, Parser)]
pub enum CliArgs {
    /// Run a plan
    Run(RunArgs),
    /// List available devices
    Devices { name: Option<String> },
    /// List available plans
    Plans { name: Option<String> },
}

#[derive(Debug, Parser)]
pub struct RunArgs {
    name: String,
    params: Option<String>,
}

impl RunArgs {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn parameters(&self) -> Result<Option<Value>, ()> {
        self.params
            .as_deref()
            .map(|paras| serde_json::from_str(paras))
            .transpose()
            .map_err(|_| ())
    }
}
