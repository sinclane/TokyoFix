use std::clone::Clone;
use std::fmt::{Display, Formatter};

pub trait FixEnum {
    fn value(&self) -> char;
}

pub enum EncryptMethod {
    NONE,
    PKCS,
    DES,
    PkcsDes,
    PGP_DES,
    PGP_DES_MD5,
    PEM_DES_MD5
}

impl FixEnum for EncryptMethod {
    fn value(&self) -> char {
        match self {
            EncryptMethod::NONE           => '0',
            EncryptMethod::PKCS           => '1',
            EncryptMethod::DES            => '2',
            EncryptMethod::PkcsDes        => '3',
            EncryptMethod::PGP_DES        => '4',
            EncryptMethod::PGP_DES_MD5    => '5',
            EncryptMethod::PEM_DES_MD5    => '6',
        }
    }
}

pub enum MsgType {
    HeartBeat,
    TestRequest,
    ResendRequest,
    Reject,
    SequenceReset,
    Logout,
    IndicationOfInterest,
    Advertistment,
    ExecutionReport,
    OrderCancelReject,
    Logon,
    News,
    Email,
    NewOrderSingle,
    NewOrderList,
    OrderCancelRequest,
    OrderCancelReplaceRequest ,
    OrderStatusRequest,
    Allocation,
    ListCancelRequest,
    ListExecute,
    ListStatusRequest,
    ListStatus,
    AllocationAck,
    DontKnowTrade,
    QuoteRequest,
    Quote,
    SettlementInstructions,
    MarketDataRequest,
    MarketDataSnapshotFullRefresh,
    MarketDataIncrementalRefresh,
    MarketDataRequestReject,
    QuoteCancel,
    QuoteStatusRequest,
    QuoteAcknowledgement,
    SecurityDefinitionRequest,
    SecurityDefinition,
    SecurityStatusRequest,
    SecurityStatus,
    TradingSessionStatusRequest,
    TradingSessionStatus,
    MassQuote,
    BusinessMessageReject,
    BidRequest,
    BidResponse,
    ListStrikePrice
}

impl FixEnum for MsgType {
    fn value(&self) -> char {
        match self {
            MsgType::HeartBeat                           =>  '0',
            MsgType::TestRequest                         =>  '1',
            MsgType::ResendRequest                       =>  '2',
            MsgType::Reject                              =>  '3',
            MsgType::SequenceReset                       =>  '4',
            MsgType::Logout                              =>  '5',
            MsgType::IndicationOfInterest                =>  '6',
            MsgType::Advertistment                       =>  '7',
            MsgType::ExecutionReport                     =>  '8',
            MsgType::OrderCancelReject                   =>  '9',
            MsgType::Logon                               =>  'A',
            MsgType::News                                =>  'B',
            MsgType::Email                               =>  'C',
            MsgType::NewOrderSingle                      =>  'D',
            MsgType::NewOrderList                        =>  'E',
            MsgType::OrderCancelRequest                  =>  'F',
            MsgType::OrderCancelReplaceRequest           =>  'G',
            MsgType::OrderStatusRequest                  =>  'H',
            MsgType::Allocation                          =>  'J',
            MsgType::ListCancelRequest                   =>  'K',
            MsgType::ListExecute                         =>  'L',
            MsgType::ListStatusRequest                   =>  'M',
            MsgType::ListStatus                          =>  'N',
            MsgType::AllocationAck                       =>  'P',
            MsgType::DontKnowTrade                       =>  'Q',
            MsgType::QuoteRequest                        =>  'R',
            MsgType::Quote                               =>  'S',
            MsgType::SettlementInstructions              =>  'T',
            MsgType::MarketDataRequest                   =>  'V',
            MsgType::MarketDataSnapshotFullRefresh       =>  'W',
            MsgType::MarketDataIncrementalRefresh        =>  'X',
            MsgType::MarketDataRequestReject             =>  'Y',
            MsgType::QuoteCancel                         =>  'Z',
            MsgType::QuoteStatusRequest                  =>  'a',
            MsgType::QuoteAcknowledgement                =>  'b',
            MsgType::SecurityDefinitionRequest           =>  'c',
            MsgType::SecurityDefinition                  =>  'd',
            MsgType::SecurityStatusRequest               =>  'e',
            MsgType::SecurityStatus                      =>  'f',
            MsgType::TradingSessionStatusRequest         =>  'g',
            MsgType::TradingSessionStatus                =>  'h',
            MsgType::MassQuote                           =>  'i',
            MsgType::BusinessMessageReject               =>  'j',
            MsgType::BidRequest                          =>  'k',
            MsgType::BidResponse                         =>  'l',
            MsgType::ListStrikePrice                     =>  'm'
        }
    }
}

impl Display for MsgType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

pub mod side {

    use std::fmt::{Display};

    pub struct Side {
        val : char
    }

    pub const BUY: Side = Side { val: '1'};
    pub const SELL: Side = Side { val: '2'};
    pub const BUY_MINUS: Side = Side { val: '3'};
    pub const SELL_PLUS: Side = Side { val: '4'};
    pub const SELL_SHORT: Side = Side { val: '5'};
    pub const SELL_SHORT_EXEMPT: Side = Side { val: '6'};
    pub const UNDISCLOSED: Side = Side { val: '7'};
    pub const CROSS: Side = Side { val: '8'};
    pub const CROSS_SHORT: Side = Side { val: '9'};

}

pub mod time_in_force {
    use std::fmt::{Display};

    pub struct TimeInForce {
        val : char
    }
    pub const DAY:TimeInForce = TimeInForce{ val: '0'};
    pub const GOOD_TILL_CANCEL:TimeInForce = TimeInForce{ val:'1' };
    pub const AT_THE_OPENING:TimeInForce = TimeInForce{ val:'2'};
    pub const IMMEDIATE_OR_CANCEL:TimeInForce = TimeInForce{ val:'3'};
    pub const FILL_OR_KILL:TimeInForce = TimeInForce{ val:'4'};
    pub const GOOD_TILL_CROSSING:TimeInForce = TimeInForce{ val:'5'};
    pub const GOOD_TILL_DATE:TimeInForce = TimeInForce{val:'6'};
}

pub mod id_source {
    use std::fmt::{Display};

    pub struct IdSource {
        val : char
    }
    pub const CUSIP:IdSource                = IdSource{val:'1'};
    pub const SEDOL:IdSource                = IdSource{val:'2'};
    pub const QUIK:IdSource                 = IdSource{val:	'3'};
    pub const ISIN:IdSource                 = IdSource{val:	'4'};
    pub const RIC:IdSource                  = IdSource{val:	'5'};
    pub const ISO_CURRENCY_CODE:IdSource    = IdSource{val:'6'};
    pub const ISO_COUNTRY_CODE:IdSource     = IdSource{val:'7'};
    pub const EXCHANGE_SYMBOL:IdSource      = IdSource{val:'8'};
    pub const CTA:IdSource                  = IdSource{val:'9'};

}

pub mod ord_status {
    use std::fmt::{Display};

    pub struct OrdStatus {
        val: char
    }

    pub const NEW: OrdStatus = OrdStatus { val: '0' };
    pub const PARTIALLY_FILLED: OrdStatus = OrdStatus { val: '1' };
    pub const FILLED: OrdStatus = OrdStatus { val: '2' };
    pub const DONE_FOR_DAY: OrdStatus = OrdStatus { val: '3' };
    pub const CANCELED: OrdStatus = OrdStatus { val: '4' };
    pub const REPLACED: OrdStatus = OrdStatus { val: '5' };
    pub const PENDING_CANCEL: OrdStatus = OrdStatus { val: '6' };
    pub const STOPPED: OrdStatus = OrdStatus { val: '7' };
    pub const REJECTED: OrdStatus = OrdStatus { val: '8' };
    pub const SUSPENDED: OrdStatus = OrdStatus { val: '9' };
    pub const PENDING_NEW: OrdStatus = OrdStatus { val: 'A' };
    pub const CALCULATED: OrdStatus = OrdStatus { val: 'B' };
    pub const EXPIRED: OrdStatus = OrdStatus { val: 'C' };
    pub const ACCEPTED_FOR_BIDDING: OrdStatus = OrdStatus { val: 'D' };
}


pub mod encrypt_method_enum {

    use std::fmt::{Display};

    pub struct EncryptMethod { val :char }

    pub const NONE: EncryptMethod        = EncryptMethod { val: '0' };
    pub const PKCS: EncryptMethod        = EncryptMethod { val: '1' };
    pub const DES: EncryptMethod         = EncryptMethod { val: '2' };
    pub const PKCS_DES: EncryptMethod    = EncryptMethod { val: '3' };
    pub const PGP_DES: EncryptMethod     = EncryptMethod { val: '4' };
    pub const PGP_DES_MD5: EncryptMethod = EncryptMethod { val: '5' };
    pub const PEM_DES_MD5: EncryptMethod = EncryptMethod { val: '6' };

}


