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

pub struct FixSessionHandler {

    from_socket_rx: mpsc::Receiver<ApplicationMessage>,
    to_msg_hdlr_tx: mpsc::Sender<FixMessage>,
    alarm_rx      : mpsc::Receiver<AlarmMessage>
}

pub struct FixMessage {
    header   : String,
    body     : String,
    trailer  : String,
    msg_type : char
}

impl FixMessage {
    pub fn get_msg_type(&self) -> char {
        self.msg_type
    }
    pub fn get_body(&self) -> String {
        self.body.clone()
    }
}

impl FixMessage {

    fn dummy(message: &String, msg_type: char) -> Self {
        Self {
            header   : String::from(""),
            body     : message.to_string(),
            trailer  : String::from(""),
            msg_type
        }
    }
    // |-----header1------|-----------------header2-----------------------------------|---body----|-trlr-|
    // |
    // 8=FIX.4.2^9=77^35=A^34=0^49=TEST_CLIENT^56=TEST_SERVER^52=20250119-16:13:08.931^98=0^108=30^10=217^
    fn new(message: &String) -> Self {

        let mut indices = Vec::new();

        for i in 0..message.len() {
            if message.chars().nth(i).unwrap()== '' {
                indices.push(i);
            }
        }

        let header_end_idx    = *indices.get(2).unwrap();
        let trailer_start_idx = *indices.get(indices.len()-2).unwrap();
        let trailer_end_idx   = *indices.get(indices.len()-1).unwrap();

        let hd = &message[0..header_end_idx];
        let bd = &message[header_end_idx..trailer_start_idx];
        let tr = &message[trailer_start_idx..trailer_end_idx];

        let mt = message.as_bytes()[header_end_idx-1] as char;

        Self {
            header   : hd.to_string(),
            body     : bd.to_string(),
            trailer  : tr.to_string(),
            msg_type : mt
        }
    }
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
        let mut flag = true;
        let mut sflag = true;
        loop {
            // New probably don't need the component anymore - but keeping it around for now as it feels
            // like I should be splitting the session layer and message handling layer into separate
            // responsibilities. To be revisited.
            let x = self.from_socket_rx.try_recv();

            let outstanding_from_socket = self.to_msg_hdlr_tx.max_capacity() - self.to_msg_hdlr_tx.capacity();
            let outstanding_from_socket = self.from_socket_rx.len();

            if outstanding_from_socket > 0 {
                if flag == true {
                    flag = false;
                    fix_println!("SK_TX: {} messages in incoming Q from socket. {} ",outstanding_from_socket, flag);

                }
            }

            if outstanding_from_socket > 0 {
                if sflag == true {
                    sflag = false;
                    fix_println!("MH_TX: {} messages in outgoing Q to msg_handler. {} ",outstanding_from_socket, flag);

                }
            }

            match x {
                Ok(app_msg) => {

                    let fix_msg = FixMessage::new(&app_msg.get_message());
                    fix_println!("SH2MK: Attempting to send msg");
                    let x2 = self.to_msg_hdlr_tx.send(fix_msg).await;
                    fix_println!("SH2MK: sent msg to MH.{}",self.to_msg_hdlr_tx.capacity());
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
                    fix_println!("Attempting to send HB");
                    let hb = &String::from("35=1");
                    let fix_hb_msg = FixMessage::dummy(hb, '1');
                    fix_println!("SH2MK: Attempting to hb msg");
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

    pub async fn run(&mut self) {
        loop {
            // New probably don't need the component anymore - but keeping it around for now as it feels
            // like I should be splitting the session layer and message handling layer into separate
            // responsibilities. To be revisited.
            let x = self.from_socket_rx.recv().await;
            match x {
                Some(app_msg) => {
                    fix_println!("Attempting to send msg");
                    let fix_msg = FixMessage::new(&app_msg.get_message());
                    let x2 = self.to_msg_hdlr_tx.send(fix_msg).await;
                    match x2 {
                        Ok(_) => {}
                        Err(em) => { fix_println!("Session Handler: error sending to Msg Handler: {}", em) }
                    };
                }
                _ => {}
            };

            let y = self.alarm_rx.recv().await;
            match y {
                Some(_) => {
                    fix_println!("Attempting to send HB");
                    let hb = &String::from("35=1");
                    let fix_hb_msg = FixMessage::dummy(hb, '1');

                    let res = self.to_msg_hdlr_tx.send(fix_hb_msg).await;

                    match res {
                        Ok(_) => {}
                        Err(em) => { fix_println!("Session Handler: error sending to Msg Handler: {}",em) }
                    };
                }
                _ => {}
            };
        }
    }
}

#[allow(dead_code)]
pub trait FixMsgHandler {
    fn on_heartbeat(&mut self, message: String);
    fn on_session_level_reject(&mut self);
    fn on_dont_know(&mut self);
    fn on_new_order_single(&mut self);
    fn on_accepted(&mut self);
    fn on_acknowledged(&mut self);
    fn on_cancel_request(&mut self);
    fn on_cancel_accepted(&mut self);
    fn on_cancel_rejected(&mut self);
    fn on_cxl_replace_request(&mut self);
    fn on_cxl_replace_accepted(&mut self);
    fn on_cxl_replace_rejected(&mut self);
    fn on_execution_report(&mut self);
}
