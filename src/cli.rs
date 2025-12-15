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
        /// Defer the pause until the next checkpoint
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
    Devices {
        /// Show information for a specific devices instead of listing all
        name: Option<String>,
    },
    /// List available plans
    Plans {
        /// Show information for a specific plan instead of listing all
        name: Option<String>,
    },
    /// Inspect or restart the environment
    Env {
        /// Reload the current environment
        #[clap(short, long)]
        reload: bool,
        /// Seconds to wait for the reload
        #[clap(short, long, requires = "reload", default_value = "10")]
        timeout: Option<u64>,
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
    /// The name of the plan to run
    name: String,
    /// The instrument session with which this plan should be associated
    #[clap(short, long, env = "BLUEAPI_INSTRUMENT_SESSION")]
    instrument_session: String,
    /// Parameters to pass to the plan in JSON format
    params: Option<String>,
    /// Run the plan in the foreground blocking until the plan is complete
    #[clap(short, long)]
    foreground: bool,
    /// Run the plan in the background returning before the plan is complete
    #[clap(short, long, overrides_with = "foreground")]
    _background: bool,
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

    pub fn foreground(&self) -> bool {
        match (self.foreground, self._background) {
            (false, false) => true, // default if neither given
            (true, true) => true,   // shouldn't be possible but foreground wins
            (true, false) => true,  // explicit --foreground passed
            (false, true) => false, // explicit --background passed
        }
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
