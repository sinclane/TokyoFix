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

    pub fn dummy(message: &String, msg_type: char) -> Self {
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
    pub fn new(message: &String) -> Self {

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