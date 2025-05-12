use std::collections::HashMap;

use clap::Parser;
use cli::{CliArgs, RunArgs};
use entities::{Device, DeviceList, PlanList, TaskReference};
use serde::de::DeserializeOwned;
use serde_json::Value;
use ureq::Agent;
use ureq::http::{Uri, uri};

mod cli;
mod entities;

fn main() {
    let args = CliArgs::parse();
    let client: Agent = Agent::config_builder()
        .http_status_as_error(false)
        .build()
        .into();
    let client = Client {
        agent: client,
        host: Uri::from_static("http://localhost:8000"),
    };
    match args {
        CliArgs::Run(run_args) => client.run_plan(run_args),
        CliArgs::Devices { name: filter } => client.get_devices(filter),
        CliArgs::Plans { name } => client.get_plans(name),
    }
}

struct Client {
    agent: Agent,
    host: Uri,
}

impl Client {
    fn request_path(&self, path: &str) -> Uri {
        uri::Builder::from(self.host.clone())
            .path_and_query(path)
            .build()
            .unwrap()
    }

    fn run_plan(&self, args: RunArgs) {
        let req = self
            .agent
            .post(self.request_path("/tasks"))
            .send_json(HashMap::from([
                ("name".to_owned(), Value::String(args.name().into())),
                ("params".to_owned(), args.parameters().unwrap().unwrap()),
            ]));
        let task = req
            .unwrap()
            .body_mut()
            .read_json::<TaskReference>()
            .unwrap();
        let mut resp = self
            .agent
            .put(self.request_path("/worker/task"))
            .send_json(task)
            .unwrap();
        println!("{resp:#?}");
        println!("{:?}", resp.body_mut().read_to_string());
    }

    fn get_devices(&self, name: Option<String>) {
        match name {
            Some(name) => println!(
                "{:#?}",
                self.get::<Device>(self.request_path(&format!("/devices/{name}")))
            ),
            None => println!(
                "{:#?}",
                self.get::<DeviceList>(self.request_path("/devices"))
            ),
        }
    }

    fn get_plans(&self, name: Option<String>) {
        let plans = match name {
            Some(name) => vec![
                self.get(self.request_path(&format!("/plans/{name}")))
                    .unwrap(),
            ],
            None => {
                self.get::<PlanList>(self.request_path("/plans"))
                    .unwrap()
                    .plans
            }
        };
        for plan in plans {
            println!("{}", plan.name,);
            println!("{}", plan.description.as_deref().unwrap_or("???"));
        }
    }
    fn get<T: DeserializeOwned>(&self, url: Uri) -> Result<T, ureq::Error> {
        self.agent.get(url).call().unwrap().body_mut().read_json()
    }
}
