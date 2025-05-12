use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Message {
    Progress(ProgressEvent),
    Worker(WorkerEvent),
    Data {
        #[serde(flatten)]
        event: Event,
    },
}

#[derive(Debug, Deserialize)]
pub struct ProgressEvent {
    task_id: String,
    statuses: HashMap<String, StatusView>,
}

#[derive(Debug, Deserialize)]
pub struct StatusView {
    display_name: String,
    current: Option<f64>,
    initial: Option<f64>,
    target: Option<f64>,
    unit: Option<String>,
    precision: Option<i32>,
    #[serde(default)]
    done: bool,
    percentage: Option<f64>,
    time_elapsed: Option<f64>,
    time_remaining: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct WorkerEvent {
    state: WorkerState,
    task_status: Option<TaskStatus>,
    #[serde(default)]
    errors: Vec<String>,
    #[serde(default)]
    warnings: Vec<String>,
}
impl WorkerEvent {
    pub(crate) fn complete(&self) -> bool {
        self.task_status.as_ref().is_some_and(|st| st.task_complete)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum WorkerState {
    Idle,
    Running,
    Pausing,
    Paused,
    Halting,
    Stopping,
    Aborting,
    Suspending,
    Panicked,
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct TaskStatus {
    task_id: Uuid,
    task_complete: bool,
    task_failed: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "name", content = "doc")]
pub enum Event {
    Stop {
        data_type: Option<Value>,
        exit_status: ExitStatus,
        #[serde(default)]
        num_events: HashMap<String, i32>,
        reason: Option<String>,
        run_start: Uuid,
        time: f64,
        uid: Uuid,
    },
    Start {
        #[serde(default)]
        data_groups: Vec<String>,
        data_session: Option<String>,
        group: Option<String>,
        owner: Option<String>,
        project: Option<String>,
        sample: Option<SampleInfo>,
        scan_id: Option<u32>,
        time: f64,
        uid: Uuid,
    },
    Event {
        uid: Uuid,
        time: f64,
        data: Value,
        timestamps: Value,
        seq_num: u32,
        descriptor: Uuid,
    },
    Descriptor {
        #[serde(default)]
        configuration: HashMap<String, Configuration>,
        data_keys: HashMap<String, DataKey>,
        name: Option<String>,
        #[serde(default)]
        object_keys: HashMap<String, Value>,
        run_start: Uuid,
        time: f64,
        uid: Uuid,
    },
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
    #[serde(default)]
    data: HashMap<String, Value>,
    #[serde(default)]
    data_keys: HashMap<String, DataKey>,
    #[serde(default)]
    timestamps: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
pub struct DataKey {
    #[serde(default)]
    choices: Vec<String>,
    #[serde(default)]
    dims: Vec<String>,
    dtype: DataType,
    dtype_numpy: Option<Value>,
    external: Option<String>,
    // limits: Option<Limits>,
    object_name: Option<String>,
    precision: Option<i32>,
    shape: Vec<Option<i32>>,
    source: String,
    units: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    String,
    Number,
    Array,
    Boolean,
    Integer,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum SampleInfo {
    Info(HashMap<String, Value>),
    Link(Uuid),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExitStatus {
    Success,
    Abort,
    Fail,
}
