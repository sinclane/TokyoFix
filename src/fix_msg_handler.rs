use std::collections::HashMap;
use crate::fix_println;
use crate::fix_session_handler::FixMsgHandler;
use std::io::Write;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use crate::countdown_actor::ResetMessage;
use crate::fix_42::tags;

pub struct MyFixMsgHandler {

    interval_tx : mpsc::Sender<u64>,
    fix_status : FixStatus
}

struct FixStatus {
    logon_complete : bool,
    next_seq_id_to_send : i32,
    next_seq_id_ro_recv : i32
}

impl FixStatus {
    fn new() -> FixStatus {  FixStatus { logon_complete :  false, next_seq_id_to_send : 0, next_seq_id_ro_recv: 0 }  }
}

impl MyFixMsgHandler {
    pub fn new(interval_sender : Sender<u64>) -> Self {
        Self {interval_tx : interval_sender, fix_status : FixStatus::new()}

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

    fn on_heartbeat(&self) {
        fix_println!("Received a Heartbeat message");
        //todo!()
    }

    fn on_logon_request(&self, message: String) {
        //todo!()
        fix_println!("Received a logon Request");

        let mut hmap = HashMap::new();

        parse_fix_message(message.as_str(), &mut hmap);

        let heartbeat_interval:u64 = hmap.get(tags::HEARTBT_INT.id()).unwrap().parse().unwrap();

        fix_println!("Remote side has requested an HB interval of {} seconds.", heartbeat_interval);

        //make sure the future is waited for so something happens.
        fix_println!("Initiating HB.");

        // TODO: There has to be a nicer way of doing this - don't want to have to clone it each time I call it
        // Perhaps: x.blocking_send(heartbeat_interval * 1000).expect("TODO: panic message");
        let x = self.interval_tx.clone();
        tokio::spawn( async move { x.send(heartbeat_interval * 1000).await.expect("TODO: panic message")});
    }

    fn on_test_request(&self) {
        todo!()
    }

    fn on_session_level_reject(&self) {
        todo!()
    }

    fn on_dont_know(&self) {
        todo!()
    }

    fn on_new_order_single(&self) {
        todo!()
    }

    fn on_accepted(&self) {
        todo!()
    }

    fn on_acknowledged(&self) {
        todo!()
    }

    fn on_cancel_request(&self) {
        todo!()
    }

    fn on_cancel_accepted(&self) {
        todo!()
    }

    fn on_cancel_rejected(&self) {
        todo!()
    }

    fn on_cxl_replace_request(&self) {
        todo!()
    }

    fn on_cxl_replace_accepted(&self) {
        todo!()
    }

    fn on_cxl_replace_rejected(&self) {
        todo!()
    }

    fn on_execution_report(&self) {
        todo!()
    }
}