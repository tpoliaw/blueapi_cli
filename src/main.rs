use clap::Parser;
use cli::{CliArgs, RunArgs};
use entities::{Device, DeviceList, PlanList};
use serde::de::DeserializeOwned;
use serde_json::Value;
use ureq::SendBody;

mod cli;
mod entities;

fn main() {
    let args = CliArgs::parse();
    match args {
        CliArgs::Run(run_args) => run_plan(run_args),
        CliArgs::Devices { name: filter } => get_devices(filter),
        CliArgs::Plans { name } => get_plans(name),
    }
}

fn run_plan(args: RunArgs) {
    let req = ureq::post("http://localhost:8000/tasks").send(
        SendBody::from_json(&Value::Object(
            [
                ("name".to_owned(), Value::String(args.name().into())),
                ("params".to_owned(), args.parameters().unwrap().unwrap()),
            ]
            .into_iter()
            .collect(),
        ))
        .unwrap(),
    );
    match req {
        Ok(mut resp) => {
            println!("content: {:?}", resp.body_mut().read_to_string());
        }
        Err(e) => {
            println!("err: {e}");
        }
    }
}

fn get_devices(name: Option<String>) {
    match name {
        Some(name) => println!(
            "{:#?}",
            get::<Device>(format!("http://localhost:8000/devices/{name}"))
        ),
        None => println!("{:#?}", get::<DeviceList>("http://localhost:8000/devices")),
    }
}

fn get_plans(name: Option<String>) {
    let plans = match name {
        Some(name) => vec![get(format!("http://localhost:8000/plans/{name}")).unwrap()],
        None => {
            get::<PlanList>("http://localhost:8000/plans")
                .unwrap()
                .plans
        }
    };
    for plan in plans {
        println!("{}", plan.name,);
        println!("{}", plan.description.as_deref().unwrap_or("???"));
    }
}

fn get<T: DeserializeOwned>(url: impl Into<String>) -> Result<T, ureq::Error> {
    ureq::get(url.into()).call().unwrap().body_mut().read_json()
}
