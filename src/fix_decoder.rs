use std::collections::HashMap;
use std::io::Error;
use std::io::ErrorKind::InvalidData;
use bytes::{BytesMut};
use tokio_util::codec;
use tokio_util::codec::{Decoder};
use std::io::{self,Write};
use crate::fix_println;

const FIX_SEPARATOR: u8        = b'';
pub(crate) struct MyFIXDecoder {
    header1 : Vec<u8>,
    header2 : Vec<u8>,
    trailer : Vec<u8>
}

impl MyFIXDecoder {
    pub(crate) fn new(config : &HashMap<String,String> ) -> Self {

        let version = config.get("version").unwrap();
        let mut hdr1 = String::new();

        if version == "4.2" {
            hdr1.push_str("8=FIX.4.29=");
        } else if version == "4.4" {
            hdr1.push_str("8=FIX.4.49=");
        } else if version == "5.0" {
            hdr1.push_str("8=FIX.5.09=");
        } else {
            println!("Unsupported version {}, defaulting to 4.2", version);
            hdr1.push_str("8=FIX.4.29=");
        }

        let sender_comp_id = config.get("sender_comp_id").unwrap();
        let target_comp_id = config.get("target_comp_id").unwrap();
        let now = chrono::offset::Utc::now().format("%Y%m%d%H%M%S%3f").to_string();
        let hdr2= format!("35=_49={sender_comp_id}56={target_comp_id}52={now}34=");

        Self {
            header1: Vec::from(hdr1),
            header2: Vec::from(hdr2),
            trailer: Vec::from("10=000")
        }
    }
}
impl Decoder for MyFIXDecoder {

    type Item = String;

    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {

        //Check in the first 12 bytes to see if tags 8= & 9= are present.
        //63 bytes is the smallest valid FIX message
        if src.len() >= 63  {
            if src[0..12].eq_ignore_ascii_case(&self.header1) {
                let mut length_sz: usize = 0;
                let mut length: usize = 0;

                //Now fetch the body length from tag 9= and see if that many bytes are available
                let mut i = 12;

                while src[i] != FIX_SEPARATOR && i < src.len() {
                    // read the next byte as a digit
                    if src[i].is_ascii_digit() {
                        length = (length * 10) + (src[i] as usize - 0x30);
                        length_sz += 1;
                    } else {
                        //todo: we found a non-digit in the length field throw exception.
                        return Err(Error::new(InvalidData, format!("non-digit found in tag 9 <body_length>.")));
                    }
                    i += 1;
                }

                //so if x = (12 + length_sz + 1 + length[min:43] + 7) bytes are present we have a valid fix msg  ... maybe.
                let msg_end = 20 + length_sz + length;

                if src.len() >= msg_end {

                    let msg = &src.iter().as_slice()[0..msg_end];

                    // validate the checksum.
                    let d1 = src[msg_end - 4];
                    let d2 = src[msg_end - 3];
                    let d3 = src[msg_end - 2];

                    if d1.is_ascii_digit() &&  d1.is_ascii_digit() && d1.is_ascii_digit() {

                        let n1 = d1 as usize - 0x30;
                        let n2 = d2 as usize - 0x30;
                        let n3 = d3 as usize - 0x30;
                        //let cksum = (n1 * 100) + (n2 * 10) + n3;

                        // copy the data out of the buffer and into the heap
                        // todo: fix this, its not right.
                        let frame = msg.to_owned().to_vec();
                        //src.advance(msg_end);
                        //src.split_to(msg_end);
                        //return the frame to the caller.
                        return Ok(Some(String::from_utf8_lossy(msg).to_string()));

                    } else {
                        return Err(Error::new(InvalidData, "Likely incorrect tag9 value"))
                    }
                } else { return Ok(None); }
            } else { return Ok(None); }
        }
        Ok(None)
    }
}
