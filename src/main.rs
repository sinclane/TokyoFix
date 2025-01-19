mod countdown_actor;
mod socket_actor;
mod fix_decoder;
mod fix_session_handler;
mod fix_msg_handler;
mod fix_42;

use crate::fix_session_handler::FixMsgHandler;
use crate::countdown_actor::{AlarmMessage, ResetMessage};
use config::{Config, File};
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex};
use tokio_util::codec::Decoder;
use crate::fix_decoder::MyFIXDecoder;
use crate::fix_session_handler::FixSessionHandler;
use crate::fix_msg_handler::MyFixMsgHandler;

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

    let decoder_impl = Arc::new(Mutex::new(MyFIXDecoder::new(&settings)));
    let decoder_clone = Arc::clone(&decoder_impl);

    let msg_handler_impl = Arc::new(MyFixMsgHandler::new());
    let msg_handler_clone = Arc::clone(&msg_handler_impl);

    let callback_impl = Arc::new(Mutex::new(FixSessionHandler::new(msg_handler_clone)));
    let callback_clone = Arc::clone(&callback_impl);

    let sa_task = tokio::spawn(async move {

        let mut sa = socket_actor::SocketActor::new(_socket, alarm_rx, interval_tx, reset_tx, decoder_clone, callback_clone);
        fix_println!("Starting SocketActor.");
        sa.start().await;
    });
    let hb_task = tokio::spawn(async move {
        let mut hb = countdown_actor::CountdownActor::new(alarm_tx, interval_rx, reset_rx);
        fix_println!("Starting CountdownActor.");
        hb.start().await;
    });

    let _ = tokio::join!(sa_task, hb_task);

    Ok(())
}
