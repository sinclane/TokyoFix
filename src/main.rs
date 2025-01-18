mod socket_actor;
mod countdown_actor;
use std::fmt::Debug;
use tokio::net::{TcpListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc};
use crate::countdown_actor::{AlarmMessage, ResetMessage};
use std::io::{self,Write};

/// Custom `print!` macro that adds a timestamp to log messages
#[macro_export]
macro_rules! fix_println {
    ($($arg:tt)*) => {{
        let timestamp = chrono::Local::now().format("%H:%M:%S.%6f").to_string();
        let filename = file!(); // Get the current file name
        let formatted_msg = format!("[{}] [{}] {}", timestamp, filename, format_args!($($arg)*));
        println!("{}", formatted_msg);
        std::io::stdout().flush().unwrap();
    }};
}

#[tokio::main]
async fn main() -> io::Result<()>{

    let port = "8080";
    let host_ip = "127.0.0.1";
    fix_println!("Started server on: {}:{}",host_ip,port);

    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    let (interval_tx, interval_rx) = mpsc::channel::<u64>(8);
    let (alarm_tx, alarm_rx) = mpsc::channel::<AlarmMessage>(8);
    let (reset_tx, reset_rx) = mpsc::channel::<ResetMessage>(8);
    let (_socket, _) = listener.accept().await?;

    let mut hb = countdown_actor::CountdownActor::new(alarm_tx, interval_rx, reset_rx);
    let mut sa = socket_actor::SocketActor::new(_socket, alarm_rx, interval_tx, reset_tx);

    let sa_task = tokio::spawn(async move { println!("Starting SocketActor."); sa.start().await;});
    let hb_task = tokio::spawn(async move { println!("Starting HBActor."); hb.start().await;});

    let _ = tokio::join!(sa_task, hb_task);

    Ok(())
}
