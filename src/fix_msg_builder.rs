use chrono::DateTime;
use crate::fix_42;
use crate::fix_42::*;
use crate::fix_42::attribute_enums::*;
use crate::fix_42::value_types::FixTag;

const FIX_FIELD_SEPARATOR: u8 = 0x01;

fn create_fix_header(buf:&mut String, length:usize, seq_no:&i32, msg_type: fix_42::attribute_enums::MsgType) {


    //"8=FIX.4.2|9=74|35=0"
    let mut tmp:String = String::from("");

    add_char_field(&mut tmp, tags::MSG_TYPE, msg_type.value());
    add_seqnum_field(&mut tmp, tags::MSG_SEQ_NO,*seq_no);
    add_string_field(&mut tmp, tags::SENDER_COMP_ID, "TEST_SERVER");
    add_string_field(&mut tmp, tags::TARGET_COMP_ID, "TEST_CLIENT");
    add_timestamp_field(&mut tmp, tags::SENDING_TIME, chrono::offset::Utc::now());

    add_string_field(buf, tags::BEGIN_STRING, "FIX.4.2");
    add_length_field(buf, tags::BODY_LENGTH, length + tmp.len());

    buf.push_str(&tmp);
}

fn create_fix_trailer(buf:&mut String)  {

    add_checksum_field(buf, tags::CHECK_SUM, generate_check_sum(buf));
}

pub fn create_fix_heartbeat(buf:&mut String, seq_no:&i32, test_request_id: &str){

    let mut tmp:String = String::from("");

    if test_request_id.len() > 0 {
        add_string_field(&mut tmp, tags::TEST_REQ_ID, test_request_id);
    }
    // Then once complete calculate the overall length using the body length as input
    // and prepend to the start of the msg.
    create_fix_header(buf, tmp.len(), seq_no, attribute_enums::MsgType::HeartBeat);

    buf.push_str(&tmp);
    // Finally calculate the checksum as append it the end
    create_fix_trailer(buf);
}

fn create_fix_logon(buf:&mut String, seq_no:&i32, hb_interval: i32, encryption_method :attribute_enums::EncryptMethod, hb_internal: usize) {
    let mut tmp: String = String::from("");

    add_char_field(&mut tmp, tags::ENCRYPT_METHOD, encryption_method.value());
    add_int_field(&mut tmp, tags::HEARTBT_INT, hb_interval);

    // Then once complete calculate the overall length using the body length as input
    // and prepend to the start of the msg.
    create_fix_header(buf, tmp.len(), seq_no, attribute_enums::MsgType::Logon);

    buf.push_str(&tmp);
    // Finally calculate the checksum as append it the end
    create_fix_trailer(buf);
}

fn create_fix_test_request(buf:&mut String, seq_no:&i32) {

    let mut tmp: String = String::from("");
    // Then once complete calculate the overall length using the body length as input
    // and prepend to the start of the msg.
    create_fix_header(buf, tmp.len(), seq_no, attribute_enums::MsgType::TestRequest);
    add_string_field(&mut tmp, tags::TEST_REQ_ID, chrono::offset::Utc::now().format("%Y%m%d%H%M%S%3f").to_string().as_str());
    buf.push_str(&tmp);
    // Finally calculate the checksum as append it the end
    create_fix_trailer(buf);
}

fn add_checksum_field(buf:&mut String, tag :FixTag, cksum:usize){
    buf.push_str(tag.id());
    buf.push('=');
    buf.push_str(format!("{:03}",cksum).as_str());
    buf.push('');
}

fn add_timestamp_field(buf:&mut String, tag :FixTag, timestamp:DateTime<chrono::offset::Utc>){
    buf.push_str(tag.id());
    buf.push('=');
    buf.push_str(timestamp.format("%Y%m%d-%H:%M:%S%.3f").to_string().as_str());
    buf.push('');
}
fn add_seqnum_field(buf:&mut String, tag: FixTag, seq_num: i32) {
    buf.push_str(tag.id());
    buf.push('=');
    buf.push_str(seq_num.to_string().as_str());
    buf.push('');
}

fn add_char_field(buf:&mut String, tag : FixTag, value : char) {
    buf.push_str(tag.id());
    buf.push('=');
    buf.push(value);
    buf.push('');
}

fn add_int_field(buf:&mut String, tag :FixTag, value : i32) {
    buf.push_str(tag.id());
    buf.push('=');
    buf.push_str(value.to_string().as_str());
    buf.push('');
}

/*fn add_enum_field(buf:&mut String, tag :FixTag, value : Fix42::FixEnum) {
    buf.push_str(tag.id());
    buf.push('=');
    buf.push(value.value());
    buf.push('');
}*/

fn add_price_field(buf:&mut String, tag :FixTag, value :f64) {
    buf.push_str(tag.id());
    buf.push('=');
    buf.push_str(value.to_string().as_str());
    buf.push('');
}

fn add_length_field(buf:&mut String, tag :FixTag, value :usize) {
    buf.push_str(tag.id());
    buf.push('=');
    buf.push_str(value.to_string().as_str());
    buf.push('');
}

fn add_qty_field(buf:&mut String, tag :FixTag, value :f64) {
    buf.push_str(tag.id());
    buf.push('=');
    buf.push_str(value.to_string().as_str());
    buf.push('');
}

fn add_string_field(buf:&mut String, tag :FixTag, value :&str) {
    buf.push_str(tag.id());
    buf.push('=');
    buf.push_str(value);
    buf.push('');
}

pub fn generate_check_sum(buf:&str) -> usize {

    let b = buf.as_bytes();

    let mut cks :usize = 0;

    for i in 0..buf.len() {
        let y = b[i] as usize;
        cks = cks + y;
    }

    cks % 256
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_generate_check_sum_0() {
        assert_eq!(generate_check_sum("8=FIX.4.49=5835=049=BuySide56=SellSide34=352=20190605-12:29:20.259"), 172);
    }

    #[test]
    fn test_generate_check_sum_1() {
        assert_eq!(generate_check_sum("8=FIX.4.29=7435=034=049=TEST_SENDER56=TEST_TARGET52=20241228-17:10:29.938112=test"), 57);
    }

    #[test]
    fn test_generate_check_sum_2() {
        assert_eq!(generate_check_sum("8=FIX.4.29=15435=D34=57849=LEH_LZJ0252=20100302-22:50:3456=CCG115=LZJ11=NF0040/0302201054=138=100055=IOC40=244=49.3859=01=ABC123ZYX21=1207=N47=A111=0"), 121);
    }
    #[test]
    fn test_generate_check_sum_3() {
        assert_eq!(generate_check_sum("8=FIX.4.29=7235=149=BuySide56=SellSide34=252=20190605-16:56:17.419112=TestReqID"), 213);
    }

    #[test]
    fn test_generate_check_sum_4() {
        assert_eq!(generate_check_sum("8=FIX.4.29=535=0"), 161);
    }

    #[test]
    fn test_add_enum_field() {

        let mut msg = String::from("8=FIX.4.49=58");
        add_char_field(&mut msg, tags::MSG_TYPE, attribute_enums::MsgType::HeartBeat.value());
        assert_eq!(msg,"8=FIX.4.49=5835=0");
    }

    #[test]
    fn test_add_string_field() {

        let mut msg = String::from("8=FIX.4.49=58");
        add_char_field(&mut msg, tags::MSG_TYPE, attribute_enums::MsgType::HeartBeat.value());
        assert_eq!(msg,"8=FIX.4.49=5835=0");
    }

    #[test]
    fn test_create_fix_heartbeat() {
        let mut msg = String::from("");
        let s = chrono::offset::Utc::now();
        create_fix_heartbeat(&mut msg, 0, "test");
        let e = chrono::offset::Utc::now();
        println!("Duration:{}",e-s);
        println!("{msg}");
    }

    #[test]
    fn test_create_fix_logon() {
        let mut msg = String::from("");

        let s = chrono::offset::Utc::now();
        create_fix_logon(&mut msg, 0, 10, attribute_enums::EncryptMethod::NONE,30);
        let e = chrono::offset::Utc::now();
        println!("Duration:{}",e-s);
        println!("{msg}");
    }
    #[test]
    fn test_create_fix_test_request() {
        let mut msg = String::from("");

        let s = chrono::offset::Utc::now();
        create_fix_test_request(&mut msg, 0);
        let e = chrono::offset::Utc::now();
        println!("Duration:{}",e-s);
        println!("{msg}");
    }
}
