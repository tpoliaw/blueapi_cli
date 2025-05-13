#![allow(unused)]
use std::collections::HashMap;

use data_model::EventDocument;
use serde::Deserialize;
use uuid::Uuid;

mod data_model;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Message {
    Progress(ProgressEvent),
    Worker(WorkerEvent),
    Data {
        task_id: Uuid,
        #[serde(flatten)]
        event: EventDocument,
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
