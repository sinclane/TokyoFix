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
use tokio::net::{TcpListener,TcpStream};
use tokio::runtime::Handle;
use tokio::sync::{mpsc, Mutex};
use tokio_util::codec::Decoder;
use crate::fix_decoder::MyFIXDecoder;
use crate::fix_session_handler::FixSessionHandler;
use crate::fix_msg_handler::MyFixMsgHandler;
use crate::socket_actor::ApplicationMessage;
use std::env;


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
async fn main() {
    // Option 1
    // --------
    // Gather all conf files from conf/ manually

    let args: Vec<String> = env::args().collect();

    println!("Sourcing parameters from: {}", args[1]);

    let settings = Config::builder()
        // File::with_name(..) is shorthand for File::from(Path::new(..))
        .add_source(File::with_name(args[1].as_str()))
        .build()
        .unwrap();

    // Print out our settings (as a HashMap)
    let settings_map = settings.try_deserialize::<HashMap<String, String>>().unwrap();

    let metrics = Handle::current().metrics();
    let n = metrics.num_workers();
    fix_println!("Runtime is using {} workers", n);
    fix_println!("\n{:?} \n\n-----------",settings_map);

    let (interval_tx, interval_rx)  = mpsc::channel::<u64>(1);
    let (alarm_tx, alarm_rx)        = mpsc::channel::<AlarmMessage>(1);
    let (reset_tx, reset_rx)        = mpsc::channel::<ResetMessage>(1);
    let (mh2sc_tx, mh2sc_rx)        = mpsc::channel::<ApplicationMessage>(3);
    let (sc2mh_tx, sc2mh_rx)        = mpsc::channel::<ApplicationMessage>(1);

    let hb_task = tokio::spawn(async move {
        let mut hb = countdown_actor::CountdownActor::new(alarm_tx, interval_rx, reset_rx);
        fix_println!("Starting CountdownActor.");
        hb.start().await;
    });

    let instance_type = settings_map.get("type").unwrap().clone();

    let sa_task = if instance_type == "server" {

        fix_println!("Starting as server");

        let port = settings_map.get("server_port").unwrap();
        let local_addr = format!("localhost:{}", port);
        fix_println!("Started server on: {}", local_addr);

        let listener = TcpListener::bind(local_addr).await.unwrap();

        let socket = match listener.accept().await {
            Ok((socket, _)) => { socket }
            Err(e) => panic!("{}", e)
        };

        fix_println!("Connection received from:{}", socket.peer_addr().unwrap());

        let decoder_impl = Arc::new(Mutex::new(MyFIXDecoder::new(&settings_map)));
        let decoder_clone = Arc::clone(&decoder_impl);
        let sa_interval_tx_clone = interval_tx.clone();
        tokio::spawn(async move {
            let mut sa = socket_actor::SocketActor::new(socket, sa_interval_tx_clone, mh2sc_rx, reset_tx, decoder_clone, sc2mh_tx);
            fix_println!("Starting SocketActor.");
            sa.run_with_try().await;
        })
    } else {

        fix_println!("Starting as client");

        let port = settings_map.get("target_port").unwrap();
        let host = settings_map.get("target_host").unwrap();
        let target_destination = format!("{}:{}", host, port);
        fix_println!("Attempting to connect to remote server on: {}", target_destination);
        let mut socket = TcpStream::connect("localhost:8080").await.unwrap();

        let decoder_impl = Arc::new(Mutex::new(MyFIXDecoder::new(&settings_map)));
        let decoder_clone = Arc::clone(&decoder_impl);
        let sa_interval_tx_clone = interval_tx.clone();

         tokio::spawn(async move {
            let mut sa = socket_actor::SocketActor::new(socket, sa_interval_tx_clone, mh2sc_rx, reset_tx, decoder_clone, sc2mh_tx);
            fix_println!("Starting SocketActor.");
            sa.run_with_try().await;
        })
    };

    let mh_interval_tx_clone = interval_tx.clone();
    let mh_task = tokio::spawn(async move {
        let mut mh: MyFixMsgHandler = MyFixMsgHandler::new(mh_interval_tx_clone, sc2mh_rx, mh2sc_tx, alarm_rx);
        fix_println!("Starting MyFixMsgHandler.");

        if instance_type == "client" {
            mh.create_and_send_logon().await;
        }

        mh.run_with_try().await;
    });

    // Use the '?' as the top-level main returns a result - so it can deal
    // with any bad result created here.

    let metrics = Handle::current().metrics();
    let n = metrics.num_alive_tasks();
    println!("Runtime is using {} num_alive_tasks", n);
    let _ = tokio::join!(hb_task, mh_task, sa_task);
}
