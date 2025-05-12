use std::collections::HashMap;

use clap::Parser;
use cli::{CliArgs, RunArgs};
use entities::{Device, DeviceList, PlanList, TaskReference};
use reqwest::Url;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::runtime::Runtime;

mod cli;
mod entities;

fn main() {
    let args = CliArgs::parse();

    let client = reqwest::Client::new();
    let client = Client {
        agent: client,
        host: Url::parse("http://localhost:8000").unwrap(),
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
        let resp = self
            .agent
            .put(self.endpoint("/worker/task"))
            .json(&task)
            .send()
            .await
            .unwrap();
        println!("{resp:#?}");
        println!("{:?}", resp.text().await)
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

    async fn get<T: DeserializeOwned>(&self, url: Url) -> Result<T, reqwest::Error> {
        self.agent.get(url).send().await.unwrap().json().await
    }

    fn endpoint(&self, path: &str) -> Url {
        self.host.join(path).unwrap()
    }
}
