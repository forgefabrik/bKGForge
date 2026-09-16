use bkgforge_browser::BrowserRuntime;
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
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let command = arguments.first().map_or("status", String::as_str);
    match command {
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
        "browser" => browser_command(&arguments[1..]),
        "help" | "--help" | "-h" => {
            println!("Usage: bkgforge-cli [status|doctor|browser]");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("unknown command: {command}\nUsage: bkgforge-cli [status|doctor|browser]");
            ExitCode::from(2)
        }
    }
}

fn browser_command(arguments: &[String]) -> ExitCode {
    match arguments.first().map_or("doctor", String::as_str) {
        "doctor" => {
            let endpoint = std::env::var("BKGFORGE_CDP_ENDPOINT")
                .unwrap_or_else(|_| "http://127.0.0.1:9222".into());
            match BrowserRuntime::connect(&endpoint) {
                Ok(runtime) => {
                    println!(
                        "browser_runtime: available\nmode: connected\nproduct: {}\ncdp_endpoint: {endpoint}",
                        runtime.version().product
                    );
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    println!(
                        "browser_runtime: unavailable\ncdp_endpoint: {endpoint}\nreason: {error}"
                    );
                    ExitCode::from(1)
                }
            }
        }
        "launch" => {
            let executable = arguments.get(1).map_or("chromium", String::as_str);
            match BrowserRuntime::launch(executable, 9222) {
                Ok(mut runtime) => {
                    println!(
                        "browser_runtime: launched\nproduct: {}",
                        runtime.version().product
                    );
                    let _ = runtime.shutdown();
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("browser launch failed: {error}");
                    ExitCode::from(1)
                }
            }
        }
        _ => {
            eprintln!("Usage: bkgforge-cli browser [doctor|launch [chromium-path]]");
            ExitCode::from(2)
        }
    }
}
