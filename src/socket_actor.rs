
//Telling the compiler I have other modules that are part of this create that need to be complied.
use std::time::SystemTime;
use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc};
use std::io::{self,Write};
use crate::countdown_actor::AlarmMessage;
use crate::countdown_actor::ResetMessage;
use crate::fix_println;

pub struct SocketActor {
    socket: TcpStream,
    alarm_rx: mpsc::Receiver<AlarmMessage>,
    interval_tx: mpsc::Sender<u64>,
    reset_tx: mpsc::Sender<ResetMessage>
}

impl SocketActor {
    pub fn new(socket: TcpStream, channel: mpsc::Receiver<AlarmMessage>, hb_channel: mpsc::Sender<u64>, reset_sender : mpsc::Sender<ResetMessage>) -> Self {
        Self {
            socket,
            alarm_rx: channel,
            interval_tx: hb_channel,
            reset_tx: reset_sender
        }
    }

    pub async fn start(&mut self) {

        let mut sent = false;
        fix_println!("Starting SocketActor");

        let mut buf = BytesMut::with_capacity(1024 * 128);

        fix_println!("Connection received from:{}", self.socket.peer_addr().unwrap());

        loop {

            let n = match self.socket.try_read_buf(&mut buf) {
               // socket closed
               Ok(n) if n == 0 => {0},
               Ok(n) => {n},
               Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {0}
               Err(e) => {
                   eprintln!("failed to read from socket; err = {:?}", e);
                   return;
               }
            };

            if (n > 0 ) {
                fix_println!("Got:{}",String::from_utf8_lossy(&buf).to_string())
                //todo:call a onRead() callBack
            }

            // Assuming we get some bytes did we get enough for a whole fix message
            // If we got a whole fix message what msg did we get
            // If we got a logon request and we are not already logged on then start logon
            // From the message extract the HB interval
            // Tell the HBer how many seconds to send a HB prompt

            if sent == false {
                self.interval_tx.send(3000).await.unwrap();
                sent = true;
            }

            //todo:is there a drain function or receive all ?
            if let Some(response) = self.alarm_rx.recv().await {

            //    println!("{}:SA: Alarm received", chrono::offset::Utc::now().format("%H:%M:%S.%3f").to_string().as_str());

                //todo:write HBmessage into buf
                let hb = "8=FIX.4.29=7435=034=049=TEST_SENDER56=TEST_TARGET52=20241228-17:10:29.938112=test";
                Self::generate_check_sum(hb);
                self.socket.write_all(hb.as_bytes()).await.expect("TODO: panic message");

                fix_println!("Sending:{}",hb);
                self.reset_tx.try_send(ResetMessage::Reset);
            }

            //todo: if I have bytes to write, write them
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