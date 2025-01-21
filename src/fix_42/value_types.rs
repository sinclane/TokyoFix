#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum ValueType {
    Int,
    DayOfMonth,
    Float,
    Qty,
    Price,
    PriceOffset,
    Amt,
    Char,
    Boolean,
    String,
    MultipleValueString,
    Currency,
    Country,
    Exchange,
    UTCTimestamp,
    UTCTimeOnly,
    LocalMktDate,
    UTCDate,
    MonthYear,
    XmlData,
    Data
}

/*impl Clone for ValueType {
    fn clone(&self) -> Self { *self}
}*/
#[allow(dead_code)]
pub const INT:ValueType   = ValueType::Int;
pub const LENGTH:ValueType   = ValueType::Int;
pub const SEQ_NO:ValueType   = ValueType::Int;
pub const TAGNUM:ValueType   = ValueType::Int;
pub const XID:ValueType   = ValueType::Int;
pub const NUM_IN_GROUP:ValueType = ValueType::Int;
pub const XIDREF:ValueType   = ValueType::Int;
pub const DAY:ValueType   = ValueType::DayOfMonth;
pub const FLOAT:ValueType = ValueType::Float;
pub const PERCENTAGE:ValueType = ValueType::Float;
pub const QTY:ValueType   = ValueType::Qty;
pub const PRICE:ValueType = ValueType::Price;
pub const PRICE_OFFSET:ValueType = ValueType::PriceOffset;
pub const AMT:ValueType = ValueType::Amt;
pub const CHAR:ValueType = ValueType::Char;
pub const BOOLEAN:ValueType = ValueType::Boolean;
pub const STRING:ValueType = ValueType::String;
pub const LANGUAGE:ValueType = ValueType::String;
pub const MULTIPLE_CHAR_VALUE:ValueType = ValueType::String;
pub const MULTIPLE_STRING_VALUE:ValueType = ValueType::MultipleValueString;
pub const CURRENCY:ValueType = ValueType::Currency;
pub const ISO_COUNTRY:ValueType = ValueType::Country;
pub const EXCHANGE:ValueType = ValueType::Exchange;
pub const UTC_TIMESTAMP:ValueType = ValueType::UTCTimestamp;
pub const TZ_TIMESTAMP:ValueType = ValueType::UTCTimestamp;
pub const UTC_TIME_ONLY:ValueType = ValueType::UTCTimeOnly;
pub const UTC_DATE_ONLY:ValueType = ValueType::UTCTimeOnly;
pub const TZ_TIME_ONLY:ValueType = ValueType::UTCTimeOnly;
pub const LOCAL_MKT_DATE:ValueType = ValueType::LocalMktDate;
pub const LOCAL_MKT_TIME:ValueType = ValueType::LocalMktDate;
pub const UTC_DATE:ValueType = ValueType::UTCDate;
pub const MONTH_YEAR:ValueType = ValueType::MonthYear;
pub const DATA:ValueType = ValueType::Data;
pub const XML_DATA:ValueType = ValueType::XmlData;
pub struct FixTag {
    pub(crate) id: &'static str, pub(crate) datatype: ValueType }

impl FixTag {
    pub fn new(id: &'static str, datatype: ValueType) -> FixTag {
        Self { id, datatype }
    }
    pub fn id(&self) -> &'static str { self.id }

    pub fn datatype(&self) -> ValueType { self.datatype }
}