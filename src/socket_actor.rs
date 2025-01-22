
//Telling the compiler I have other modules that are part of this create that need to be complied.

use bytes::BytesMut;
use tokio::io::{AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use std::io::{self,Write};
use std::sync::Arc;
use tokio_util::codec::{Decoder};
use crate::countdown_actor::AlarmMessage;
use crate::countdown_actor::ResetMessage;
use crate::fix_decoder::MyFIXDecoder;
use crate::fix_println;


pub struct SocketActor {
    socket: TcpStream,
    alarm_rx: mpsc::Receiver<AlarmMessage>,
    interval_tx: mpsc::Sender<u64>,
    writable_rx: mpsc::Receiver<ApplicationMessage>,
    reset_tx: mpsc::Sender<ResetMessage>,
    decoder: Arc<Mutex<dyn Decoder<Item = String, Error = std::io::Error> + Send + Sync>>,
    callback: Arc<Mutex<dyn SocketActorCallback + Send + Sync>>
}

pub struct ApplicationMessage {
    message: String
}

impl ApplicationMessage {

    pub fn new(message: String) -> ApplicationMessage { ApplicationMessage { message } }
}

pub trait SocketActorCallback {
    fn on_message_rx(&mut self, message: String);
    fn on_alarm_rx(&mut self, message: String);
}

// Try to avoid Socket Actor knowing anything about the message structure/protocol.
// Hence decoder is passed in as a dyn
impl SocketActor {
    pub fn new(socket:       TcpStream,
               channel:      mpsc::Receiver<AlarmMessage>,
               hb_channel: mpsc::Sender<u64>,
               msg_channel:  mpsc::Receiver<ApplicationMessage>,
               reset_sender: mpsc::Sender<ResetMessage>,
               decoder:      Arc<Mutex<dyn Decoder<Item = String, Error = std::io::Error> + Send + Sync>>,
               callback:     Arc<Mutex<dyn SocketActorCallback + Send + Sync>> ) -> Self {
        Self {
            socket,
            alarm_rx:    channel,
            interval_tx: hb_channel,
            writable_rx: msg_channel,
            reset_tx:    reset_sender,
            decoder,
            callback
        }
    }

    pub async fn start(&mut self) {

        let mut sent = false;
        fix_println!("Starting SocketActor");

        let mut buf = BytesMut::with_capacity(1024 * 128);

        let mut decoder = self.decoder.lock().await; // Lock the decoder for mutable access

        fix_println!("Connection received from:{}", self.socket.peer_addr().unwrap());

        loop {

            let num_bytes = match self.socket.try_read_buf(&mut buf) {
               Ok(num_bytes) => { if num_bytes == 0 { break; } else {num_bytes} },
               Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {0}
               Err(e) => {
                   eprintln!("failed to read from socket; err = {:?}", e);
                   return;
               }
            };

            if num_bytes > 0 {
                //todo:call a onRead() callBack
                let Some(x) = decoder.decode(&mut buf).unwrap() else { todo!() };
                // a single callback for now but it could be a list of callbacks I guess.
                // todo perhaps implement the timer reset as a callback rather than a channel ?
                let callback = self.callback.lock().await.on_message_rx(x);
                //self.callback.onMessage_rx(x).await;
            }

            if let Some(_) = self.alarm_rx.recv().await {
                // This should cause a Response to be written in to the application Message mpsc channel
                let callback = self.callback.lock().await.on_alarm_rx(String::from("TBD"));
                /*
                let hb = "8=FIX.4.29=7435=034=049=TEST_SENDER56=TEST_TARGET52=20241228-17:10:29.938112=test";
                Self::generate_check_sum(hb);
                self.socket.write_all(hb.as_bytes()).await.expect("TODO: panic message");
                //todo: handle this gracefully . Perhaps just drop out of loop ( can we check reason code ).
                fix_println!("Sending:{}",hb);
                self.reset_tx.try_send(ResetMessage::Reset).unwrap();
                */
            }

            if let Some(writable) = self.writable_rx.recv().await {

                let msg = writable.message.as_bytes();

                //todo: need some buffer, store, queue to append the created messages to
                //      the try_write many write multiple messages or just a bit of one
                //      don't assume anything ...
                let num_bytes = match self.socket.try_write(msg) {
                    Ok(num_bytes) => { if num_bytes == 0 { break; } else { num_bytes } },
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => { 0 }
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                if num_bytes > 0 {

                    //todo: this tells the Countdown Timer to reset itself as a message has been /is being written
                    //      This is part of the FIX protocal and so should prob be moved up a layer.
                    //      Need to make sure that all the messages are sequence properly.
                    self.reset_tx.try_send(ResetMessage::Reset).unwrap();
                }
            }
        }
    }

    pub fn generate_check_sum(buf: &str) -> usize {
        let b = buf.as_bytes();

        let mut cks: usize = 0;

        for i in 0..buf.len() {
            let y = b[i] as usize;
            cks = cks + y;
        }

        cks % 256
    }
}