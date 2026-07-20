use std::process::Child;
use crate::create_server;

use super::prelude::*;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio;
use serde_json::{Map, Value};
use fantoccini::{ ClientBuilder, Client };

pub async fn setup(create_entities: bool, enable_inline_editing: bool) -> Result<(tokio::task::JoinHandle<()>, Child, Client), Box<dyn std::error::Error>> {
    // Create and start the Actix-web server
    let server_task = tokio::spawn(async move {
        let db = setup_db(create_entities).await;
        create_server!(db, false, None, enable_inline_editing);
    });

    // Ensure a clean geckodriver: kill any pre-existing instance (e.g. a
    // developer-started daemon or a leftover session from a previous test
    // that panicked before teardown) and start a fresh one. Geckodriver
    // can only host a single WebDriver session at a time, and re-using a
    // stale session yields "SessionNotCreated: Session is already started".
    let _ = Command::new("pkill")
        .args(["-x", "geckodriver"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    // Give the OS a moment to release the port.
    tokio::time::sleep(Duration::from_millis(200)).await;

    let geckodriver = Command::new("geckodriver")
            .stdout(Stdio::null()) // Redirect stdout to /dev/null
            .stderr(Stdio::null()) // Redirect stderr to /dev/null
            .spawn()?;

    // Wait until geckodriver is accepting connections on :4444.
    for _ in 0..50 {
        if std::net::TcpStream::connect("127.0.0.1:4444").is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // run headless firefox
    let mut caps = Map::new();
    let mut firefox_options = Map::new();
    let args = vec![Value::String("-headless".to_string())];
    firefox_options.insert("args".to_string(), Value::Array(args));
    caps.insert(
        "moz:firefoxOptions".to_string(),
        Value::Object(firefox_options),
    );

    let c = ClientBuilder::native()
        .capabilities(caps)
        .connect("http://localhost:4444")
        .await
        .expect("failed to connect to WebDriver");

    Ok((server_task, geckodriver, c))
}

pub async fn teardown(server_task: tokio::task::JoinHandle<()>, mut geckodriver: Child, c: Client) ->  Result<(), fantoccini::error::CmdError> {
    let res = c.close().await;
    let _ = geckodriver.kill().expect("Failed to stop geckodriver");
    let _server = server_task.abort();
    res
}