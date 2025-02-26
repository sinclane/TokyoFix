use std::collections::HashMap;
use crate::{fix_msg_builder, fix_println};
use crate::fix_message::FixMessage;
use std::io::Write;
use std::iter::Skip;
use std::slice::Iter;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Sender, Receiver};
use tokio::sync::mpsc::error::TryRecvError;
use tokio::task::yield_now;
use crate::countdown_actor::{AlarmMessage, ResetMessage};
use crate::fix_42::attribute_enums::{EncryptMethod, FixEnum, MsgType};
use crate::fix_42::{attribute_enums, tags};
use crate::socket_actor::ApplicationMessage;


struct FixMsgStore {

    store : Vec<ApplicationMessage>
}

impl FixMsgStore {
    fn new() -> FixMsgStore {
       Self {
           store : Vec::new()
       }
    }
    fn push(&mut self, value : ApplicationMessage) {
        self.store.push(value);
    }
    fn get_single(&self, sequence_number: i32) -> &ApplicationMessage {
        // Probably want an iterator here
        // as well send to send replays in batches, yielding once the buffer is full
        // What is a sensible amount of data to send - do we send message at a time or just a mass
        // also potentially want to throttle resend msgs/sec ( for extra credit )
        return self.store.get(sequence_number as usize).unwrap();
    }

    fn iter(&self, n :usize) -> Skip<Iter<'_, ApplicationMessage>> {
        return self.store.iter().skip(n);
    }

    fn get_slice(&self, begin :usize, end :usize) -> &[ApplicationMessage] {
        return &self.store[begin..end];
    }

    fn len(&self) -> usize {
        return self.store.len();
    }
}
pub struct MyFixMsgHandler {

    interval_tx : Sender<u64>,
    app_msg_rx  : Receiver<ApplicationMessage>,
    app_msg_tx  : Sender<ApplicationMessage>,
    alarm_rx    : Receiver<AlarmMessage>,
    fix_status  : FixStatus,
    msg_store   : FixMsgStore
}

struct FixStatus {
    logon_complete : bool,
    next_seq_id_to_send : i32,
    next_seq_id_ro_recv : i32,
    hb_interval : u64,
    status : FixSessionStatus
}

enum FixServerStatus {
    AWAITING_LOGON,
    LOGON_RECEIVED,
}

enum FixClientStatus {
    UNSENT,
    LOGON_SENT,
}

#[derive(PartialEq, Eq)]
enum FixSessionStatus {
    DOWN,
    UP,
}

impl FixStatus {
    fn new() -> FixStatus {
        FixStatus {
            logon_complete      :  false,
            next_seq_id_to_send : 0,
            next_seq_id_ro_recv : 0,
            hb_interval         : 10,
            status              : FixSessionStatus::DOWN
        }
    }
}

impl MyFixMsgHandler {

    pub fn new(interval_sender : Sender<u64>, app_msg_rx: Receiver<ApplicationMessage>, app_msg_sender : Sender<ApplicationMessage>, alarm_rx :Receiver<AlarmMessage> ) -> Self {
        Self {
            interval_tx: interval_sender,
            app_msg_rx,
            app_msg_tx: app_msg_sender,
            fix_status: FixStatus::new(),
            msg_store: FixMsgStore::new(),
            alarm_rx
        }
    }

    async fn handle_fix_message(&mut self, msg: &FixMessage) {

        if msg.get_msg_type() == MsgType::Logon.value() {
            fix_println!("Calling: on_logon");
            self.on_logon_request(msg.get_body()).await;
        } else if msg.get_msg_type() == MsgType::TestRequest.value() {
            fix_println!("Calling: on_test_request");
            self.on_test_request(msg.get_body()).await;

        } else if msg.get_msg_type() == MsgType::HeartBeat.value() {
            fix_println!("Calling: on_heartbeat");
            self.on_heartbeat(msg.get_body());

        } else if msg.get_msg_type() == MsgType::SequenceReset.value() {
            fix_println!("Resetting Sequence Numbers.");

        } else {
            fix_println!("Unknown message type:'{}'",msg.get_msg_type());
        }
    }

    pub async fn run_with_try(&mut self) {

        fix_println!("Start Msg handler loop.");

        loop {

            let recvd = self.app_msg_rx.try_recv();

            match recvd {
                Err(TryRecvError::Empty) => {},
                Err(TryRecvError::Disconnected) => fix_println!("MH_RX: something went wrong."),
                Ok(app_msg) => {
                    //fix_println!("Received from Socket: {}", app_msg.get_message());
                    let fix_msg = FixMessage::new(&app_msg.get_message());
                    self.handle_fix_message(&fix_msg).await;
                }
            };

            let x = self.alarm_rx.try_recv();
            match x {
                Ok(_) => {
                    self.create_and_send_heartbeat("").await;
                }
                Err(TryRecvError::Empty) => {},
                Err(TryRecvError::Disconnected) => { },
            };

            yield_now().await;
        }
    }

    async fn resend(&self, message : ApplicationMessage) {

        let res = self.app_msg_tx.send(message).await;

        match res {
            Ok(_) =>  {},
            Err(e) => {fix_println!("Error sending FIX msg to socket handler {}",e);}
        }
        fix_println!("There are {} messages in ths inbound store",self.msg_store.len());
    }
    async fn send(&mut self, message : ApplicationMessage) {

        let store_message = message.clone();

        let res = self.app_msg_tx.send(message).await;

        match res {
            Ok(_) =>  { self.msg_store.push(store_message);},
            Err(e) => {fix_println!("Error sending FIX msg to socket handler {}",e);}
        }
        fix_println!("There are {} messages in ths inbound store",self.msg_store.len());
    }
    // e.g. "8=FIX.4.29=7435=034=049=TEST_SENDER56=TEST_TARGET52=20241228-17:10:29.938112=test";
    async fn create_and_send_heartbeat(&mut self, test_request_id: &str) {

        let mut hb = String::new();
        fix_msg_builder::create_fix_heartbeat(&mut hb, self.fix_status.next_seq_id_to_send, test_request_id);
        self.fix_status.next_seq_id_to_send += 1;
        let msg = ApplicationMessage::new(hb);

        self.send(msg).await;
    }

    pub async fn create_and_send_logon(&mut self) {

        let mut logon = String::new();

        fix_msg_builder::create_fix_logon(&mut logon, self.fix_status.next_seq_id_to_send, self.fix_status.hb_interval, EncryptMethod::NONE);

        self.fix_status.next_seq_id_to_send += 1;

        let msg = ApplicationMessage::new(logon);
        let res = self.app_msg_tx.send(msg).await;

        self.fix_status.status = FixSessionStatus::UP;

        match res {
            Ok(_) => {},
            Err(e) => {fix_println!("Error sending FIX msg to socket handler {}",e);}
        }
    }

    fn on_heartbeat(&mut self, message:String) {
        //parse message
        //Update last ping time
        //Update next expected sequence number
    }
    async fn on_resend_request(&mut self, message: String) {

        let mut hmap = HashMap::new();
        parse_fix_message(message.as_str(), &mut hmap);

        let begin_sq_no:u64 = hmap.get(tags::BEGIN_SEQ_NO.id()).unwrap().parse().unwrap();
        let end_sq_no:u64   = hmap.get(tags::END_SEQ_NO.id()).unwrap().parse().unwrap();

        for message in self.msg_store.get_slice(begin_sq_no as usize, end_sq_no as usize) {

            self.resend(message.clone()).await;
        }
    }

    async fn on_test_request(&mut self, message: String) {
        self.create_and_send_heartbeat(&*message).await;
    }

    fn on_session_level_reject(&mut self) {
        todo!()
    }

    fn on_dont_know(&mut self) {
        todo!()
    }

    async fn on_logon_request(&mut self, message: String) {

        fix_println!("Received a logon Request");

        let mut hmap = HashMap::new();
        parse_fix_message(message.as_str(), &mut hmap);

        let heartbeat_interval:u64 = hmap.get(tags::HEARTBT_INT.id()).unwrap().parse().unwrap();

        self.fix_status.hb_interval = heartbeat_interval;
        fix_println!("Remote side has requested an HB interval of {} seconds.", heartbeat_interval);

        //make sure the future is waited for so something happens.
        fix_println!("Initiating HB.");

        // TODO: There has to be a nicer way of doing this - don't want to have to clone it each time I call it
        let x = self.interval_tx.clone();

        tokio::spawn( async move { x.send(heartbeat_interval * 1000).await.expect("TODO: panic message")});

        //This is the initial response to the logon request
        //
        if( self.fix_status.status == FixSessionStatus::DOWN ) {
            self.create_and_send_logon().await;
        }
    }
}
pub fn parse_fix_message(buf:&str, hmap:&mut HashMap<String, String>)  {

    // split the string by the SOH character
    let x = buf.split('');
    //todo: remove the last entry
    for s in x {

        if !s.is_empty() {
            //println!("Processing attribute: {}",s);

            //then split each of those by the  '=' character
            let o = s.find('=');
            match o {
                Some(result) => {
                    // use the lhs as the key and the rhs as the value
                    hmap.insert(s[..result].to_string(), s[result + 1..].to_string());
                }
                None => {
                    println!("Badly formed attribute found: {}", s);
                }
            }
        }
    }
}

impl FixMsgHandler for MyFixMsgHandler {

    fn on_new_order_single(&mut self) {
        todo!()
    }

    fn on_accepted(&mut self) {
        todo!()
    }

    fn on_acknowledged(&mut self) {
        todo!()
    }

    fn on_cancel_request(&mut self) {
        todo!()
    }

    fn on_cancel_accepted(&mut self) {
        todo!()
    }

    fn on_cancel_rejected(&mut self) {
        todo!()
    }

    fn on_cxl_replace_request(&mut self) {
        todo!()
    }

    fn on_cxl_replace_accepted(&mut self) {
        todo!()
    }

    fn on_cxl_replace_rejected(&mut self) {
        todo!()
    }

    fn on_execution_report(&mut self) {
        todo!()
    }
}

pub trait FixMsgHandler {

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