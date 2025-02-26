
//Telling the compiler I have other modules that are part of this create that need to be complied.

use bytes::BytesMut;
use tokio::io::{AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use std::io::{self,Write};
use std::sync::Arc;
use tokio::sync::mpsc::error::{TryRecvError};
use tokio::task::yield_now;
use tokio_util::codec::{Decoder};
use crate::countdown_actor::AlarmMessage;
use crate::countdown_actor::ResetMessage;
use crate::fix_decoder::MyFIXDecoder;
use crate::fix_println;


pub struct SocketActor {
    socket:      TcpStream,
    interval_tx: mpsc::Sender<u64>,
    from_mh_rx:  mpsc::Receiver<ApplicationMessage>,
    reset_tx:    mpsc::Sender<ResetMessage>,
    decoder:     Arc<Mutex<dyn Decoder<Item = String, Error = std::io::Error> + Send + Sync>>,
    to_sh_tx:  mpsc::Sender<ApplicationMessage>
}

pub struct ApplicationMessage {
    message: String,
    bmsg: Vec<u8>
}

impl Clone for ApplicationMessage {
    fn clone(&self) -> Self {
        ApplicationMessage {
            message : self.message.clone(),
            bmsg : self.bmsg.clone()
        }
    }
}

impl ApplicationMessage {

    pub fn new(message: String) -> ApplicationMessage {
        ApplicationMessage {
            message,
            bmsg : Vec::new()
        }
    }
    pub fn get_message(&self) -> &String { &self.message }
}

// Try to avoid Socket Actor knowing anything about the message structure/protocol.
// Hence decoder is passed in as a dyn
impl SocketActor {
    pub fn new(socket:       TcpStream,

               hb_channel:     mpsc::Sender<u64>,
               from_mh_rx:     mpsc::Receiver<ApplicationMessage>,
               reset_sender:   mpsc::Sender<ResetMessage>,
               decoder:        Arc<Mutex<dyn Decoder<Item = String, Error = std::io::Error> + Send + Sync>>,
               to_sh_tx:       mpsc::Sender<ApplicationMessage>) -> Self {
        Self {
            socket,
            interval_tx: hb_channel,
            from_mh_rx,
            reset_tx:    reset_sender,
            decoder,
            to_sh_tx
        }
    }

    pub async fn run_with_try(&mut self) {

        fix_println!("Running SocketActor");
        let mut buf = BytesMut::with_capacity(1024 * 128);
        let mut decoder = self.decoder.lock().await; // Lock the decoder for mutable access

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
                let z = buf.clone();
                let Some(x) = decoder.decode(&mut buf).unwrap() else { todo!() };
                let y = x.clone();
                let am = ApplicationMessage::new(x);
                fix_println!("Read {} bytes from socket:{}",num_bytes,String::from_utf8_lossy(z.as_ref()));
                fix_println!("Read from socket2:{}",y);
                let res = self.to_sh_tx.send(am).await;
                match res {
                    Ok(_) => {},
                    Err(e) => {eprintln!("failed to send to Session Handler.{}",e);}
                }
            }

            let result =  self.from_mh_rx.try_recv();
            match result {
                //Err(TryRecvError::Empty) => { if tried % 200000 == 0 { fix_println!("MH->SK: tried {} times so far",tried);}; },
                Err(TryRecvError::Empty) => { },
                Err(TryRecvError::Disconnected) => fix_println!("MH_RX: something went wrong."),
                Ok(writable) => {

                    let msg = writable.message.as_bytes();

                    //todo: need some buffer, store, queue to append the created messages to
                    //      the try_write many write multiple messages or just a bit of one
                    //      don't assume anything ...
                    let num_bytes = match self.socket.try_write(msg) {
                        Ok(num_bytes) => { if num_bytes == 0 { break; } else { fix_println!("Wrote {} bytes: {}", num_bytes, String::from_utf8_lossy(msg) ); num_bytes}},
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
            };
            yield_now().await;
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
                let am = ApplicationMessage::new(x);
                fix_println!("MH->SK: waiting @ session send, pending={}", self.from_mh_rx.len());
                let res = self.to_sh_tx.send(am).await;
                fix_println!("MH->SK: session sent, pending={}", self.from_mh_rx.len());
                match res {
                    Ok(_) => {},
                    Err(e) => {eprintln!("failed to send to Session Handler.{}",e);}
                }
            }


            fix_println!("MH->SK: about to wait @ msg recv, pending={}", self.from_mh_rx.len());
            //dbg!(&self.writable_rx);
            let result =  self.from_mh_rx.recv().await;
            fix_println!("MH->SK: msgs remaining  to recv.{}", self.from_mh_rx.len());

            match result {
                Some(writable) => {
                    let msg = writable.message.as_bytes();

                    //todo: need some buffer, store, queue to append the created messages to
                    //      the try_write many write multiple messages or just a bit of one
                    //      don't assume anything ...
                    let num_bytes = match self.socket.try_write(msg) {
                        Ok(num_bytes) => { if num_bytes == 0 { break; } else { eprintln!("Wrote {} bytes: {}", num_bytes, String::from_utf8_lossy(msg) ); num_bytes}},
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
                        fix_println!("SK:resetting HB countdown");
                        let result = self.reset_tx.try_send(ResetMessage::Reset);
                        match result {
                            Ok(_) => {},
                            Err(e) => {eprintln!("failed to reset countdown; err = {:?}", e)}
                        };
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