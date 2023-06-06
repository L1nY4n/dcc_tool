use crate::model::{DccCmd, DccPacket};

#[derive(Debug)]
pub enum ToBackend {
    Refresh,
    SetInterval(u32),
    StockAdd(String),
    StockDel(String),
    DccCmd(String, DccCmd),
}

#[derive(Debug)]
pub enum ToFrontend {
    DccPacket(String, DccPacket),
}
