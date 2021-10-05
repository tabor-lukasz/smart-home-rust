// use std::env;
use chrono::{Timelike, Utc};
use std::io;
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
// use musql_async::*;

mod temp_reader;
mod common_structs;
mod config;

use temp_reader::TempReader;
use common_structs::TempWithTs;
use config::Config;

const CONFIG_PATH: &str = "conf/conf.yaml";

fn nanos_till_next_awake() -> Duration {
    let now = Utc::now();
    let decs = now.second() * 10 + now.nanosecond() / 100000000;
    let mut diff = 300 as i64 - decs as i64;
    if diff <= 0 {
        diff += 600;
    }
    Duration::new((diff as u64) / 10, ((diff as u32) % 10) * 100000000)
}

struct MyHttpSrv {
    pub current_temp: Arc<RwLock<TempWithTs>>,
    pub config: Config,
}

impl MyHttpSrv {
    pub async fn run(&self) -> io::Result<()> {
        let listener = TcpListener::bind("0.0.0.0:22222").await?;

        log("Server started".to_string()).await;

        loop {
            tokio::select! {
                mut s = self.listen(&listener) => {
                    self.process_socket(&mut s).await
                },
                _ = Self::reader_loop(self.current_temp.clone()) => {},
            }
        }
    }

    async fn listen(&self, listener: &TcpListener) -> TcpStream {
        let (stream, _) = listener.accept().await.unwrap();
        log("Connection established!".to_string()).await;
        stream
    }

    async fn reader_loop(current: Arc<RwLock<TempWithTs>>) {
        loop {
            sleep(nanos_till_next_awake()).await;
            match read_senors().await {
                Ok(v) => {
                    *current.write().await = v;

                }
                Err(e) => log(e.to_string()).await,
            }
        }
    }

    async fn process_socket(&self, socket: &mut TcpStream) {
        let contents = format!(
            "{{\"location\": \"{}\", \"temp\": \"{}\", \"ts\": \"{:?}\"}}",
            self.config.location,
            self.current_temp.read().await.temp as f64 / 1000.0,
            self.current_temp.read().await.ts
        );

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            contents.len(),
            contents
        );

        socket.writable().await.unwrap();
        let _ = socket.try_write(response.as_bytes());
        println!("Request processing done");
    }
}

async fn log(msg: String) {
    println!("{}\t{}", Utc::now(), msg);
    let mut file = match OpenOptions::new()
        .append(true)
        .create(true)
        .open("log_rsh.txt")
        .await
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Log file open error: {}", e);
            return;
        }
    };
    if let Err(e) = file
        .write_all(format!("{}\t{}\n", Utc::now(), msg).as_bytes())
        .await
    {
        eprintln!("Log error: {}", e);
    }
}

async fn log_temp(msg: String) {
    let mut file = match OpenOptions::new()
        .append(true)
        .create(true)
        .open("log_temp.txt")
        .await
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Log temp file open error: {}", e);
            return;
        }
    };
    if let Err(e) = file
        .write_all(format!("{}\t{}\n", Utc::now(), msg).as_bytes())
        .await
    {
        eprintln!("Log temp error: {}", e);
    }
}

async fn read_senors() -> Result<TempWithTs, String> {
    match TempReader::get_temps().await {
        Ok(v) => {
            log(format!("{}", v.temp as f64 / 1000.0)).await;
            log_temp(format!("{}", v.temp as f64 / 1000.0)).await;
            Ok(v)
        }
        Err(e) => return Err(e.to_string()),
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let srv = MyHttpSrv {
        current_temp: Arc::new(RwLock::new(TempWithTs {
            temp: -273000,
            ts: Utc::now(),
        })),
        config: Config::new(std::path::Path::new(CONFIG_PATH)),
    };

    {
        log("System start".to_string()).await;
        // read_senors(&srv.current_temp).await;
    }

    let _ = tokio::spawn(async move {
        srv.run().await.unwrap();
    })
    .await;

    Ok(())
}
