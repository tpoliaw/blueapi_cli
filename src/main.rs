use std::collections::HashMap;

use clap::Parser;
use cli::{CliArgs, RunArgs};
use entities::{Device, DeviceList, PlanList, TaskId, TaskReference};
use messages::Message;
use reqwest::Url;
use rumqttc::{Event, MqttOptions, Packet, QoS};
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{self, Receiver};
use uuid::Uuid;

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
        let mut messages = self.message_stream(task.task_id).await.unwrap().unwrap();
        let resp = self
            .agent
            .put(self.endpoint("/worker/task"))
            .json(&task)
            .send()
            .await
            .unwrap();

        if resp.status().is_success() {
            while let Some(msg) = messages.recv().await {
                match &msg {
                    Message::Progress(_) => {}
                    Message::Worker(worker_event) => println!("{worker_event:#?}"),
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

    async fn message_stream(&self, task_id: TaskId) -> Option<Result<Receiver<Message>, ()>> {
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
                            Ok(evt) if evt.task_id() != Some(task_id) => continue,
                            Ok(evt) => {
                                let complete = match &evt {
                                    Message::Worker(wk) => wk.complete(),
                                    _ => false,
                                };
                                let closed = tx.send(evt).await.is_err();
                                if closed || complete {
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

    fn endpoint(&self, path: &str) -> Url {
        self.host.join(path).unwrap()
    }
}
