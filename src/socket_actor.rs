
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
    socket:      TcpStream,
    interval_tx: mpsc::Sender<u64>,
    writable_rx: mpsc::Receiver<ApplicationMessage>,
    reset_tx:    mpsc::Sender<ResetMessage>,
    decoder:     Arc<Mutex<dyn Decoder<Item = String, Error = std::io::Error> + Send + Sync>>,
    session_tx:  mpsc::Sender<ApplicationMessage>
}

pub struct ApplicationMessage {
    message: String
}

impl ApplicationMessage {

    pub fn new(message: String) -> ApplicationMessage { ApplicationMessage { message } }
    pub fn get_message(&self) -> &String { &self.message }
}

pub trait SocketActorCallback {
    fn on_message_rx(&mut self, message: String);
    fn on_alarm_rx(&mut self, message: String);
}

// Try to avoid Socket Actor knowing anything about the message structure/protocol.
// Hence decoder is passed in as a dyn
impl SocketActor {
    pub fn new(socket:       TcpStream,

               hb_channel:        mpsc::Sender<u64>,
               msg_channel:       mpsc::Receiver<ApplicationMessage>,
               reset_sender:      mpsc::Sender<ResetMessage>,
               decoder:           Arc<Mutex<dyn Decoder<Item = String, Error = std::io::Error> + Send + Sync>>,
               session_handler:   mpsc::Sender<ApplicationMessage>) -> Self {
        Self {
            socket,
            interval_tx: hb_channel,
            writable_rx: msg_channel,
            reset_tx:    reset_sender,
            decoder,
            session_tx: session_handler
        }
    }

    pub async fn run(&mut self) {

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
                let res = self.session_tx.send(ApplicationMessage::new(x)).await;
                match res {
                    Ok(_) => {eprintln!("Successfully sent to Session Handler.");},
                    Err(e) => {eprintln!("failed to send to Session Handler.{}",e);}
                }
            }

            if self.writable_rx.len() > 0 {
                eprintln!("MH->SK: msgs pending={}", self.writable_rx.len());
            }

            let result =  self.writable_rx.recv().await;
            match result {
                Some(writable) => {
                    let msg = writable.message.as_bytes();

                    //todo: need some buffer, store, queue to append the created messages to
                    //      the try_write many write multiple messages or just a bit of one
                    //      don't assume anything ...
                    let num_bytes = match self.socket.try_write(msg) {
                        Ok(num_bytes) => { if num_bytes == 0 { break; } else { eprintln!("Wrote {} bytes to socket", num_bytes ); num_bytes}},
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => { 0 },
                        Err(e) => {
                            eprintln!("failed to read from socket; err = {:?}", e);
                            return;
                        }
                    };

                    if num_bytes > 0 {

                        //todo: this tells the Countdown Timer to reset itself as a message has been /is being written
                        //      This is part of the FIX protocal and so should prob be moved up a layer.
                        //      Need to make sure that all the messages are sequence properly.
                        //self.reset_tx.try_send(ResetMessage::Reset).unwrap();
                    }
                }
                None => { println!("Couldn't get msg"); }
            };
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