mod countdown_actor;
mod socket_actor;

use crate::countdown_actor::{AlarmMessage, ResetMessage};
use config::{Config, File};
use glob::glob;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{self, Write};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

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
async fn main() -> io::Result<()> {
    // Option 1
    // --------
    // Gather all conf files from conf/ manually
    let settings = Config::builder()
        // File::with_name(..) is shorthand for File::from(Path::new(..))
        .add_source(File::with_name("config/config.toml"))
        .build()
        .unwrap();

    // Print out our settings (as a HashMap)
    let settings = settings.try_deserialize::<HashMap<String, String>>().unwrap();

    println!("\n{:?} \n\n-----------",settings);

    let port = settings.get("server_port").unwrap();
    let local_addr = format!("localhost:{}", port);
    fix_println!("Started server on: {}", local_addr);

    let listener = TcpListener::bind(local_addr).await?;

    let (interval_tx, interval_rx) = mpsc::channel::<u64>(8);
    let (alarm_tx, alarm_rx) = mpsc::channel::<AlarmMessage>(8);
    let (reset_tx, reset_rx) = mpsc::channel::<ResetMessage>(8);
    let (_socket, _) = listener.accept().await?;

    let mut hb = countdown_actor::CountdownActor::new(alarm_tx, interval_rx, reset_rx);
    let mut sa = socket_actor::SocketActor::new(_socket, alarm_rx, interval_tx, reset_tx);

    let sa_task = tokio::spawn(async move {
        fix_println!("Starting SocketActor.");
        sa.start().await;
    });
    let hb_task = tokio::spawn(async move {
        fix_println!("Starting CountdownActor.");
        hb.start().await;
    });

    let _ = tokio::join!(sa_task, hb_task);

    Ok(())
}
