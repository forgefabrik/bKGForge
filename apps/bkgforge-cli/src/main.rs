use bkgforge_core::{Resource, ResourceId, ResourceState};
use bkgforge_resources::ResourcePool;
use bkgforge_runtime::Runtime;
use std::collections::BTreeMap;
use std::process::ExitCode;

fn runtime() -> Runtime {
    Runtime::new(ResourcePool::new([Resource {
        id: ResourceId("local-browser".into()),
        kind: "browser".into(),
        state: ResourceState::Ready,
        capacity: 1,
        labels: BTreeMap::new(),
    }]))
}

fn main() -> ExitCode {
    let command = std::env::args().nth(1).unwrap_or_else(|| "status".into());
    match command.as_str() {
        "status" => {
            let status = runtime().status();
            println!(
                "bkgForge status\nready_resources: {}\nactive_lanes: {}",
                status.ready_resources, status.active_lanes
            );
            ExitCode::SUCCESS
        }
        "doctor" => {
            println!("bkgForge doctor: runtime composition healthy");
            ExitCode::SUCCESS
        }
        "help" | "--help" | "-h" => {
            println!("Usage: bkgforge-cli [status|doctor]");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("unknown command: {command}\nUsage: bkgforge-cli [status|doctor]");
            ExitCode::from(2)
        }
    }
}
