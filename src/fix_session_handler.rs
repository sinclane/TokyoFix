use crate::fix_println;
use crate::socket_actor::SocketActorCallback;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::fix_msg_handler::MyFixMsgHandler;

pub struct FixSessionHandler {
    msg_handler : Arc<dyn FixMsgHandler + Send + Sync>,
}

struct FixMsgComponents {
    header   : String,
    body     : String,
    trailer  : String,
    msg_type : char
}

impl FixMsgComponents {

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
        let trailer_start_idx = *indices.get(indices.len()-3).unwrap();
        let trailer_end_idx   = *indices.get(indices.len()-2).unwrap();

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
    pub fn new(msg_handler: Arc<dyn FixMsgHandler + Send + Sync>) -> Self {
        Self {
            msg_handler
        }
    }
}

#[allow(dead_code)]
pub trait FixMsgHandler {
    fn on_heartbeat(&mut self);
    fn on_logon_request(&mut self, message: String);
    fn on_test_request(&mut self);
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
    fn create_and_send_heartbeat(&mut self, request_id: &str);
}

impl SocketActorCallback for FixSessionHandler {
    fn on_message_rx(&mut self, message: String) {
        // todo! : split message into hdr1, hdr2, body, trailer \
        //         validate hdr2
        //         pass body onto application_msg_handler
        //         Session level messages: logon, heartbeat, testrequest should all be handled at this layer
        //         NewOrderSingle etc should be handled further up.

        let msg_as_struct = FixMsgComponents::new(&message);

        fix_println!("Handling: {}",message);

        if msg_as_struct.msg_type == '0' {
            self.msg_handler.on_heartbeat();

        } else if msg_as_struct.msg_type == 'A' {
            self.msg_handler.on_logon_request(message);

        } else if msg_as_struct.msg_type == '1' {
            fix_println!("Calling: on_test_request");

        } else {
            fix_println!("Unknown message type:{}",msg_as_struct.msg_type);
        }
    }

    fn on_alarm_rx(&mut self, message: String) {
        //todo!()
        fix_println!("Creating HB:{}",message);
        self.msg_handler.create_and_send_heartbeat("");
    }
}