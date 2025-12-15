use clap::Parser;
use serde::Serialize;
use serde_json::Value;

use crate::entities::SourceInfo;

#[derive(Debug, Parser)]
pub enum CliArgs {
    /// Run a plan
    Run(RunArgs),
    /// Pause the current task
    Pause {
        #[clap(short, long)]
        defer: bool,
    },
    /// Resume a paused task
    Resume,
    /// Stop the current task, marking any ongoing run as success
    Stop,
    /// Abort the current task, marking any ongoing run as failed
    Abort { reason: Option<String> },
    /// List available devices
    Devices { name: Option<String> },
    /// List available plans
    Plans { name: Option<String> },
    /// Inspect or restart the environment
    Env {
        #[clap(short, long)]
        reload: bool,
        #[clap(short, long, requires = "reload")]
        timeout: Option<f64>,
    },
    /// Retrieve the installed packages and their sources
    GetPythonEnv(PackageFilter),
    /// Print the current state of the worker
    State,
    /// Listen to events output by blueapi
    Listen,
}

#[derive(Debug, Parser)]
pub struct RunArgs {
    name: String,
    #[clap(short, long, env = "BLUEAPI_INSTRUMENT_SESSION")]
    instrument_session: String,
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

    pub(crate) fn instrument_session(&self) -> Value {
        (self.instrument_session.as_str()).into()
    }
}

#[derive(Debug, Serialize, Parser)]
pub struct PackageFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(short, long)]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(short, long)]
    source: Option<SourceInfo>,
}
