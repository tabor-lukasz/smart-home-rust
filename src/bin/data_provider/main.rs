
// use std::env;
use tokio::fs::{OpenOptions};
use tokio::net::TcpStream;
use tokio::time::{sleep,Duration};
use chrono::{DateTime, Timelike, Utc};
use tokio::net::TcpListener;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use std::sync::Arc;
use std::io;
// use musql_async::*;

mod temp_reader;

use temp_reader::TempReader;
// const INTERVAL_DFLT: u64 = 60;


fn nanos_till_next_awake() -> Duration {
    let now = Utc::now();
    let decs = now.second()*10 + now.nanosecond()/100000000;
    let mut diff = 300 as i64 - decs as i64;
    if diff <= 0 {
        diff += 600;
    }
    Duration::new((diff as u64)/10, ((diff as u32) % 10)*100000000)
}

struct MyHttpSrv {
    pub current_temp: Arc<RwLock<i32>>,
    last_read: Arc<RwLock<DateTime<Utc>>>,
}

impl MyHttpSrv {
    pub async fn run(&self) -> io::Result<()> {
        let listener = TcpListener::bind("0.0.0.0:22222").await?;

        log("Server started".to_string()).await;

        loop {
            tokio::select! {
                s = self.listen() => {self.process_socket(s).await},
                _ = reader_loop(self.current_temp.clone()) => {},
            }
        }
    }

    async fn listen(&self, listener: &TcpListener) -> TcpStream {
        let (stream, _) =listener.accept().await.unwrap();
        log("Connection established!".to_string()).await;
        stream
        //self.process_socket(&mut socket).await;
    }


    async fn reader_loop(current: Arc<RwLock<i32>>) {
        loop {
            sleep( nanos_till_next_awake()).await;
            match read_senors().await {
                Ok(v) => {
                    *current.write().await = v;
                },
                Err(e) => log(e.to_string()).await,
            }
        }
    }

    async fn process_socket(&self, socket: &mut TcpStream) {

        let contents = format!("{{temp: {}}}",*self.current_temp.read().await as f64 / 1000.0);

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
    println!("{}\t{}",Utc::now(), msg);
    let mut file = match OpenOptions::new().append(true).create(true).open("log_rsh.txt").await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Log file open error: {}",e);
            return;
        },
    };
    if let Err(e) = file.write_all(format!("{}\t{}\n",Utc::now(), msg).as_bytes()).await {
        eprintln!("Log error: {}",e);
    }
}

async fn log_temp(msg: String) {
    let mut file = match OpenOptions::new().append(true).create(true).open("log_temp.txt").await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Log temp file open error: {}",e);
            return;
        },
    };
    if let Err(e) = file.write_all(format!("{}\t{}\n",Utc::now(), msg).as_bytes()).await {
        eprintln!("Log temp error: {}",e);
    }
}

async fn read_senors() -> Result<i32,String> {
    match TempReader::get_temps().await {
        Ok(v) => {
            log(format!("{}",v as f64/1000.0)).await;
            log_temp(format!("{}",v as f64/1000.0)).await;
            Ok(v)
        },
        Err(e) => return Err(e.to_string()),
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let srv = MyHttpSrv{
        current_temp: Arc::new(RwLock::new(-123000)),
        last_read: Arc::new(RwLock::new(Utc::now())),
    };

    {
        log("System start".to_string()).await;
        // read_senors(&srv.current_temp).await;
    }

    let _ = tokio::spawn( async move {
        srv.run().await.unwrap();
    }).await;

    Ok(())
}
