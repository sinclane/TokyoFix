mod countdown_actor;
mod socket_actor;
mod fix_decoder;
mod fix_session_handler;
mod fix_msg_handler;
mod fix_42;
mod fix_msg_builder;
mod fix_message;

use crate::countdown_actor::{AlarmMessage, ResetMessage};
use config::{Config, File};
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::runtime::Handle;
use tokio::sync::{mpsc, Mutex};
use tokio_util::codec::Decoder;
use crate::fix_decoder::MyFIXDecoder;
use crate::fix_session_handler::FixSessionHandler;
use crate::fix_msg_handler::MyFixMsgHandler;
use crate::socket_actor::ApplicationMessage;

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

    let metrics = Handle::current().metrics();
    let n = metrics.num_workers();
    println!("Runtime is using {} workers", n);


    println!("\n{:?} \n\n-----------",settings);

    /* Client
    let port = settings.get("target_port").unwrap();
    let host = settings.get("target_host").unwrap();
    let target_destination = format!("{}:{}", host, port);
    let mut socket = TcpStream::connect("localhost:8080").await?;
     */

    //As Server
    let port = settings.get("server_port").unwrap();
    let local_addr = format!("localhost:{}", port);
    fix_println!("Started server on: {}", local_addr);
    let listener = TcpListener::bind(local_addr).await?;

    let (interval_tx, interval_rx)     = mpsc::channel::<u64>(1);
    let (alarm_tx, alarm_rx)           = mpsc::channel::<AlarmMessage>(1);
    let (reset_tx, reset_rx)           = mpsc::channel::<ResetMessage>(1);
    let (mh2sc_msg_tx, mh2sc_msg_rx)   = mpsc::channel::<ApplicationMessage>(3);
    let (sc2mh_tx, sc2mh_rx)   = mpsc::channel::<ApplicationMessage>(1);

    let hb_task = tokio::spawn(async move {
        let mut hb = countdown_actor::CountdownActor::new(alarm_tx, interval_rx, reset_rx);
        fix_println!("Starting CountdownActor.");
        hb.start().await;
    });

    let mh_interval_tx_clone = interval_tx.clone();
    let mh_task = tokio::spawn(async move {
        let mut mh  =  MyFixMsgHandler::new(mh_interval_tx_clone, sc2mh_rx, mh2sc_msg_tx, alarm_rx);
        fix_println!("Starting MyFixMsgHandler.");
        mh.run_with_try().await;
    });

    // Use the '?' as the top-level main returns a result - so it can deal
    // with any bad result created here.
    let (socket, _) = listener.accept().await?;
    fix_println!("Connection received from:{}", socket.peer_addr()?);

    let decoder_impl = Arc::new(Mutex::new(MyFIXDecoder::new(&settings)));
    let decoder_clone = Arc::clone(&decoder_impl);
    let sa_interval_tx_clone = interval_tx.clone();
    let sa_task = tokio::spawn(async move {
        let mut sa = socket_actor::SocketActor::new(socket, sa_interval_tx_clone, mh2sc_msg_rx, reset_tx, decoder_clone, sc2mh_tx);
        fix_println!("Starting SocketActor.");
        sa.run_with_try().await;
    });

    let metrics = Handle::current().metrics();
    let n = metrics.num_alive_tasks();
    println!("Runtime is using {} num_alive_tasks", n);

    let _ = tokio::join!(sa_task, hb_task, mh_task);

    Ok(())
}
