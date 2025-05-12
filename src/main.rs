use std::collections::HashMap;

use clap::Parser;
use cli::{CliArgs, RunArgs};
use entities::{Device, DeviceList, PlanList, TaskReference};
use messages::Message;
use reqwest::Url;
use rumqttc::{Event, MqttOptions, Packet, QoS};
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{self, Receiver};

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
        // mqtt: rumqttc::Client::new(MqttOptions::new("bcli", "localhost", 61613), 10),
    };

    let rt = Runtime::new().expect("Couldn't create runtime");
    rt.block_on(async {
        match args {
            CliArgs::Run(run_args) => client.run_plan(run_args).await,
            CliArgs::Devices { name: filter } => client.get_devices(filter).await,
            CliArgs::Plans { name } => client.get_plans(name).await,
        }
    });
}

struct Client {
    agent: reqwest::Client,
    host: Url,
    mqtt: (String, u16),
    // mqtt: Receiver<Message>,
}

fn event_stream(host: &str, port: u16) -> Result<Receiver<Message>, ()> {
    todo!()
}

impl Client {
    async fn run_plan(&self, args: RunArgs) {
        let req = self
            .agent
            .post(self.endpoint("/tasks"))
            .json(&HashMap::from([
                ("name".to_owned(), Value::String(args.name().into())),
                ("params".to_owned(), args.parameters().unwrap().unwrap()),
            ]))
            .send()
            .await;
        let task = req.unwrap().json::<TaskReference>().await.unwrap();
        // println!("created task");
        let mut messages = self.message_stream().await.unwrap().unwrap();
        // println!("Created message stream");
        let resp = self
            .agent
            .put(self.endpoint("/worker/task"))
            .json(&task)
            .send()
            .await
            .unwrap();
        // println!("Started task");
        // println!("{resp:#?}");
        // println!("{:?}", resp.text().await);

        while let Some(msg) = messages.recv().await {
            println!("{msg:?}");
        }
    }

    async fn get_devices(&self, name: Option<String>) {
        match name {
            Some(name) => println!(
                "{:#?}",
                self.get::<Device>(self.endpoint(&format!("/devices/{name}")))
                    .await
            ),
            None => println!(
                "{:#?}",
                self.get::<DeviceList>(self.endpoint("/devices")).await
            ),
        }
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

    async fn message_stream(&self) -> Option<Result<Receiver<Message>, ()>> {
        // println!("Creating client and connection");
        let (client, mut conn) =
            rumqttc::AsyncClient::new(MqttOptions::new("bcli", &self.mqtt.0, self.mqtt.1), 0);
        // println!("    created");
        let (tx, rx) = mpsc::channel(10);
        let _client = client.clone();
        tokio::spawn(async move {
            // println!("    Spawned ");
            let _client = _client;
            let tx = tx;
            loop {
                // println!("    looping");
                match conn.poll().await {
                    Ok(Event::Incoming(Packet::ConnAck(c))) => {
                        // println!("{c:?}");
                    }
                    Ok(Event::Incoming(Packet::Publish(data))) => {
                        // let msg = serde_json::from_slice::<Message>(&data.payload);
                        let evt = serde_json::from_slice::<Message>(&data.payload);
                        let msg = String::from_utf8(data.payload.to_vec());
                        match evt {
                            Ok(evt) => {
                                // println!("    Event: {evt:#?}");
                                let complete = match &evt {
                                    Message::Worker(wk) => wk.complete(),
                                    _ => false,
                                };
                                tx.send(evt).await.unwrap();
                                if complete {
                                    break;
                                }
                            }
                            Err(_) => {
                                // println!(
                                //     "    Error: {:#?}",
                                //     serde_json::from_slice::<Value>(&data.payload)
                                // );
                                panic!();
                            }
                        }
                        let value = serde_json::from_slice::<Message>(&data.payload);
                        // println!("Message: {msg:?}",);
                        // tx.send(msg.unwrap()).await.unwrap();
                    }
                    Ok(msg) => {
                        println!("Recv: {msg:?}");
                    }
                    Err(e) => println!("Error: {e:?}"),
                }
            }
            // println!("Ending message loop");
        });
        client
            .subscribe("public/worker/event", QoS::AtMostOnce)
            .await
            .unwrap();
        // println!("    Subscribed");

        Some(Ok(rx))
    }

    async fn get<T: DeserializeOwned>(&self, url: Url) -> Result<T, reqwest::Error> {
        self.agent.get(url).send().await.unwrap().json().await
    }

    fn endpoint(&self, path: &str) -> Url {
        self.host.join(path).unwrap()
    }
}
