use std::collections::HashMap;
use std::time::{Duration, Instant};

use clap::Parser;
use cli::{CliArgs, RunArgs};
use entities::{Device, DeviceList, PlanList, TaskReference};
use messages::Message;
use reqwest::Url;
use rumqttc::{Event, MqttOptions, Packet, QoS};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{self, Receiver};
use tokio::time;
use uuid::Uuid;

use crate::cli::PackageFilter;
use crate::entities::{EnvironmentState, NewState, PythonEnvironment, WorkerState};

mod cli;
mod entities;
mod messages;

fn main() {
    let args = CliArgs::parse();

    let client = reqwest::Client::new();
    let client = Client {
        agent: client,
        host: Url::parse("http://localhost:8000").unwrap(),
        mqtt: ("localhost".into(), 1883),
    };

    let rt = Runtime::new().expect("Couldn't create runtime");
    rt.block_on(async {
        match args {
            CliArgs::Run(run_args) => client.run_plan(run_args).await,
            CliArgs::Devices { name: filter } => client.list_devices(filter).await.unwrap(),
            CliArgs::Plans { name } => client.get_plans(name).await,
            CliArgs::Pause { defer } => client.pause(defer).await,
            CliArgs::Resume => client.resume().await,
            CliArgs::Stop => client.stop().await,
            CliArgs::Abort { reason } => client.abort(reason).await,
            CliArgs::State => client.state().await,
            CliArgs::Env { reload, timeout } => match reload {
                true => client.reload_env(timeout).await,
                false => println!("{:?}", client.get_env().await),
            },
            CliArgs::GetPythonEnv(filter) => client.get_python_env(filter).await,
            CliArgs::Listen => client.listen().await,
        }
    });
}

struct Client {
    agent: reqwest::Client,
    host: Url,
    mqtt: (String, u16),
}

impl Client {
    async fn run_plan(&self, args: RunArgs) {
        let req = self
            .agent
            .post(self.endpoint("/tasks"))
            .json(&HashMap::from([
                ("name".to_owned(), Value::String(args.name().into())),
                ("params".to_owned(), args.parameters().unwrap().unwrap()),
                ("instrument_session".into(), "cm12345-2".into()),
            ]))
            .send()
            .await;
        let task = req.unwrap().json::<TaskReference>().await.unwrap();
        let mut messages = self.message_stream().await.unwrap().unwrap();
        let resp = self
            .agent
            .put(self.endpoint("/worker/task"))
            .json(&task)
            .send()
            .await
            .unwrap();

        if resp.status().is_success() {
            while let Some(msg) = messages.recv().await {
                if msg.task_id().is_none_or(|id| id != task.task_id) {
                    continue;
                }
                match &msg {
                    Message::Progress(_) => {}
                    Message::Worker(worker_event) => {
                        println!("{worker_event:#?}");
                        if worker_event.complete() {
                            break;
                        }
                    }
                    Message::Data { event, .. } => println!("{event:#?}"),
                }
            }
        } else {
            println!("{resp:?}");
            println!("{:?}", resp.text().await);
        }
    }

    async fn list_devices(&self, name: Option<String>) -> Result<(), reqwest::Error> {
        let devices = match name {
            Some(name) => vec![
                self.get::<Device>(self.endpoint(&format!("/devices/{name}")))
                    .await?,
            ],
            None => self
                .get::<DeviceList>(self.endpoint("/devices"))
                .await?
                .into_inner(),
        };
        for device in devices {
            println!("{}", device);
        }
        Ok(())
    }

    async fn get_plans(&self, name: Option<String>) {
        let plans = match name {
            Some(name) => vec![
                self.get(self.endpoint(&format!("/plans/{name}")))
                    .await
                    .unwrap(),
            ],
            None => {
                self.get::<PlanList>(self.endpoint("/plans"))
                    .await
                    .unwrap()
                    .plans
            }
        };
        for plan in plans {
            println!("{}", plan.name,);
            println!("{}", plan.description.as_deref().unwrap_or("???"));
        }
    }

    async fn state(&self) {
        let state = self
            .get::<WorkerState>(self.endpoint("/worker/state"))
            .await
            .unwrap();
        println!("{state:?}")
    }
    async fn pause(&self, defer: bool) {
        self.set_state(WorkerState::Paused, None, Some(defer)).await
    }

    async fn resume(&self) {
        self.set_state(WorkerState::Running, None, None).await
    }

    async fn stop(&self) {
        self.set_state(WorkerState::Stopping, None, None).await
    }

    async fn abort(&self, reason: Option<String>) {
        self.set_state(WorkerState::Aborting, reason, None).await
    }

    async fn set_state(&self, new_state: WorkerState, reason: Option<String>, defer: Option<bool>) {
        self.put::<_, WorkerState>(
            self.endpoint("/worker/state"),
            &NewState {
                new_state,
                reason,
                defer,
            },
        )
        .await
        .unwrap();
    }

    async fn get_env(&self) -> EnvironmentState {
        self.get(self.endpoint("/environment")).await.unwrap()
    }

    async fn reload_env(&self, timeout: Option<f64>) {
        let old = self
            .agent
            .delete(self.endpoint("/environment"))
            .send()
            .await
            .unwrap()
            .json::<EnvironmentState>()
            .await
            .unwrap();
        let timeout = timeout.map(|t| Instant::now() + Duration::from_secs_f64(t));
        while timeout.is_none_or(|t| Instant::now() < t) {
            let env = self.get_env().await;
            if let Some(msg) = env.error_message {
                panic!("{msg}");
            }
            if env.initialized && env.environment_id != old.environment_id {
                println!("{env:?}");
                break;
            }
            time::sleep(Duration::from_millis(500)).await;
        }
        panic!("Timeout waiting for environment to reload")
    }

    async fn get_python_env(&self, filter: PackageFilter) {
        let env = self
            .agent
            .get(self.endpoint("/python_environment"))
            .query(&filter)
            .send()
            .await
            .unwrap()
            .json::<PythonEnvironment>()
            .await
            .unwrap();
        println!("Scratch enabled: {}", env.scratch_enabled);
        for pkg in env.installed_packages {
            println!("- {}", pkg);
        }
    }

    async fn listen(&self) {
        let mut messages = self.message_stream().await.unwrap().unwrap();
        while let Some(msg) = messages.recv().await {
            println!("{msg:?}");
        }
    }

    async fn message_stream(&self) -> Option<Result<Receiver<Message>, ()>> {
        let options = MqttOptions::new(
            format!("bcli-{}", Uuid::new_v4()),
            &self.mqtt.0,
            self.mqtt.1,
        );

        let (client, mut conn) = rumqttc::AsyncClient::new(options, 10);
        client
            .subscribe("public/worker/event", QoS::AtMostOnce)
            .await
            .unwrap();
        let (tx, rx) = mpsc::channel(10);
        tokio::spawn(async move {
            let tx = tx;
            loop {
                match conn.poll().await {
                    Ok(Event::Incoming(Packet::Publish(data))) => {
                        match serde_json::from_slice::<Message>(&data.payload) {
                            Ok(evt) => {
                                let closed = tx.send(evt).await.is_err();
                                if closed {
                                    break;
                                }
                            }
                            Err(e) => {
                                println!("Err: {e}\n{}", String::from_utf8_lossy(&data.payload))
                            }
                        }
                    }
                    Ok(msg) => println!("Recv: {msg:?}"),
                    Err(e) => println!("Error: {e:?}"),
                }
            }
        });

        Some(Ok(rx))
    }

    async fn get<T: DeserializeOwned>(&self, url: Url) -> Result<T, reqwest::Error> {
        self.agent.get(url).send().await.unwrap().json().await
    }

    async fn put<D: Serialize, T: DeserializeOwned>(
        &self,
        url: Url,
        data: &D,
    ) -> Result<T, reqwest::Error> {
        self.agent
            .put(url)
            .json(data)
            .send()
            .await
            .unwrap()
            .json()
            .await
    }

    fn endpoint(&self, path: &str) -> Url {
        self.host.join(path).unwrap()
    }
}
