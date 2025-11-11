#![allow(unused)]
use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;
use url::Url;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "name", content = "doc")]
pub enum EventDocument {
    Stop(Stop),
    Start(Start),
    Descriptor(Descriptor),
    Event(Event),
    Datum(Datum),
    Resource(Resource),
    EventPage(EventPage),
    DatumPage(DatumPage),
    StreamResource(StreamResource),
    StreamDatum(StreamDatum),
}

#[derive(Debug, Deserialize)]
pub struct Stop {
    pub data_type: Option<Value>,
    pub exit_status: ExitStatus,
    #[serde(default)]
    pub num_events: HashMap<String, i32>,
    pub reason: Option<String>,
    pub run_start: Uuid,
    pub time: f64,
    pub uid: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct Start {
    #[serde(default)]
    pub data_groups: Vec<String>,
    pub data_session: Option<String>,
    pub group: Option<String>,
    pub owner: Option<String>,
    pub project: Option<String>,
    pub sample: Option<SampleInfo>,
    pub scan_id: Option<u32>,
    pub time: f64,
    pub uid: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct Descriptor {
    #[serde(default)]
    pub configuration: HashMap<String, Configuration>,
    pub data_keys: HashMap<String, DataKey>,
    pub name: Option<String>,
    #[serde(default)]
    pub object_keys: HashMap<String, Value>,
    pub run_start: Uuid,
    pub time: f64,
    pub uid: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct Event {
    pub uid: Uuid,
    pub time: f64,
    pub data: Value,
    pub timestamps: Value,
    pub seq_num: u32,
    pub descriptor: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct Datum {
    pub datum_id: String,
    pub datum_kwargs: HashMap<String, Value>,
    pub resource: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct Resource {
    pub resource_kwargs: HashMap<String, Value>,
    pub resource_path: String,
    pub root: String,
    pub spec: String,
    pub uid: Uuid,
    pub path_semantics: Option<PathSemantics>,
    pub run_start: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct EventPage {
    pub data: HashMap<String, Vec<Value>>,
    pub time: Vec<f64>,
    pub timestamps: HashMap<String, Vec<Value>>,
    pub descriptor: String,
    pub seq_num: Vec<i32>,
    pub uid: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct DatumPage {
    pub datum_id: Vec<String>,
    pub datum_kwargs: HashMap<String, Vec<Value>>,
    pub resource: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct StreamResource {
    pub data_key: String,
    pub mimetype: String,
    pub parameters: HashMap<String, Value>,
    pub run_start: Option<Uuid>,
    pub uid: Uuid,
    pub uri: Url,
}

#[derive(Debug, Deserialize)]
pub struct StreamDatum {
    pub descriptor: Uuid,
    pub indices: StreamRange,
    pub seq_nums: StreamRange,
    pub stream_resource: Uuid,
    pub uid: String,
}

#[derive(Debug, Deserialize)]
pub struct StreamRange {
    start: i32,
    stop: i32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PathSemantics {
    Posix,
    Windows,
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
    limits: Option<Limits>,
    object_name: Option<String>,
    precision: Option<i32>,
    shape: Vec<Option<i32>>,
    source: String,
    units: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Limits {
    alarm: Option<LimitsRange>,
    control: Option<LimitsRange>,
    display: Option<LimitsRange>,
    hysteresis: Option<f64>,
    rds: Option<RdsRange>,
    warning: Option<LimitsRange>,
}

#[derive(Debug, Deserialize)]
pub struct RdsRange {
    time_difference: f64,
    value_difference: f64,
}

#[derive(Debug, Deserialize)]
pub struct LimitsRange {
    high: Option<f64>,
    low: Option<f64>,
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
