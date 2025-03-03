use crate::fix_println;
use crate::socket_actor::{ApplicationMessage};
use std::io::Write;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::sync::mpsc::error::TryRecvError;
use tokio::task::yield_now;
use crate::countdown_actor::{AlarmMessage, ResetMessage};
use crate::fix_msg_handler::MyFixMsgHandler;
use crate::fix_message::FixMessage;

pub struct FixSessionHandler {

    from_socket_rx: mpsc::Receiver<ApplicationMessage>,
    to_msg_hdlr_tx: mpsc::Sender<FixMessage>,
    alarm_rx      : mpsc::Receiver<AlarmMessage>
}

impl FixSessionHandler {
    pub fn new( from_socket_rx : mpsc::Receiver<ApplicationMessage> , to_msg_hdlr_tx :mpsc::Sender<FixMessage> , alarm_rx:      mpsc::Receiver<AlarmMessage>,  ) -> Self {
        Self {
            from_socket_rx,
            to_msg_hdlr_tx,
            alarm_rx
        }
    }

    pub async fn run_with_try(&mut self) {

        loop {
            // New probably don't need the component anymore - but keeping it around for now as it feels
            // like I should be splitting the session layer and message handling layer into separate
            // responsibilities. To be revisited.
            let x = self.from_socket_rx.try_recv();
            match x {
                Ok(app_msg) => {

                    let fix_msg = FixMessage::new(&app_msg.get_message());
                    let x2 = self.to_msg_hdlr_tx.send(fix_msg).await;

                    match x2 {
                        Ok(_) => {}
                        Err(em) => { fix_println!("Session Handler: error sending to Msg Handler: {}", em) }
                    };
                }
                Err(TryRecvError::Empty) => {},
                Err(TryRecvError::Disconnected) => {},
            };

            let x = self.alarm_rx.try_recv();
            match x {
                Ok(_) => {

                    let hb = &String::from("35=1");
                    let fix_hb_msg = FixMessage::dummy(hb, '1');
                    let res = self.to_msg_hdlr_tx.send(fix_hb_msg).await;

                    match res {
                        Ok(_) => {}
                        Err(em) => { fix_println!("Session Handler: error sending to Msg Handler: {}",em) }
                    };
                }
                Err(TryRecvError::Empty) => {},
                Err(TryRecvError::Disconnected) => { },
            };
            yield_now().await;
        }
    }
}


