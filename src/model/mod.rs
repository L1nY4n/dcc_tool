use bytes::{BufMut};
use std::{
    io::{self, BufReader, BufWriter, Read, Write},
    net::{TcpStream},
    time::{SystemTime, UNIX_EPOCH}
};

use crc::{Crc, CRC_16_MODBUS};
use crossbeam::{
    channel::{Receiver, Sender},
    select,
};

use crate::backend::message::ToFrontend;

pub const MODBUS: Crc<u16> = Crc::<u16>::new(&CRC_16_MODBUS);

pub struct Dcc {
    pub addr: String,
    pub socket: Option<TcpStream>,
    pub device_type: u8,
    pub device_id: u64,
    pub version: u8,
    pub io_data: IoData,

    pub sender: Option<Sender<ToFrontend>>,
}

impl Default for Dcc {
    fn default() -> Self {
        Self {
            addr: String::default(),
            socket: None,
            version: Default::default(),
            device_type: Default::default(),
            device_id: Default::default(),
            io_data: Default::default(),
            sender: None,
        }
    }
}

impl Dcc {
    pub fn init(&mut self, stream: TcpStream, sender: Sender<ToFrontend>, cmd_r: Receiver<DccCmd>) {
        eprintln!("dcc socket init");
        let mut writer = BufWriter::new((stream.try_clone().unwrap()));
        self.socket = Some(stream);
        self.sender = Some(sender);
        std::thread::spawn(move || {
            eprintln!("cmd receive");
            loop {
                select! {
                                    recv(cmd_r)->cmd =>{
                                        if let Ok(c) = cmd {
                                            eprintln!("{:?}",c);
                                            
                                            match c {
                                            DccCmd::DO(do_ctr) => {

                                            let mut pkt =  DccPacket::do_ctr(do_ctr,&0u64,&0u8);
                                            eprintln!("send: {:?}",pkt);
                                             let data = pkt.encode();
                                             eprintln!("send: {:x?}",data);
                                             eprintln!("send: {:?}",data);
                                              writer.write_all(&data).unwrap();

                                              writer.flush().unwrap()
                                            },
                                            DccCmd::AO(io) =>{
                                                let mut  pkt =  DccPacket::ao_ctr(io,&0u64,&0u8);
                                                eprintln!("send: {:?}",pkt);
                                                let data = pkt.encode();
                                                eprintln!("send: {:x?}",data);
                                                eprintln!("send: {:?}",data);
                                                writer.write_all(&data).unwrap();
                                                writer.flush().unwrap()
                                            },
                                            DccCmd::VO(io) =>{
                                                let mut  pkt =  DccPacket::vo_ctr(io,&0u64,&0u8);
                                                eprintln!("send: {:?}",pkt);
                                                let data = pkt.encode();
                                                eprintln!("send: {:x?}",data);
                                                eprintln!("send: {:?}",data);
                                                writer.write_all(&data).unwrap();
                                                writer.flush().unwrap()
                                            }
                                            DccCmd::T485 =>{

                                            },
                }
                                        }
                                    }
                                   }
            }
        });
        crossbeam::thread::scope(|scope| {
            scope.spawn(|_| {
                self.handle_connection()
                    .map_err(|e| eprintln!("Error: {}", e))
            });
        })
        .unwrap();
    }

    pub fn send(&self) {
        if let Some(stream) = &self.socket {
            let mut writer = BufWriter::new(stream);
            writer.write_all(b"test").unwrap();
        }
    }
    fn handle_connection(&mut self) -> io::Result<()> {
        let stream = self.socket.as_mut().unwrap();
        let peer_addr = stream.peer_addr().expect("Stream has peer_addr");
        eprintln!("Incoming from {}", peer_addr);
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = BufWriter::new(stream);

        loop {
            let mut rx_bytes = [0u8; 256];
            let bytes_read = reader.read(&mut rx_bytes)?;
            // eprintln!("message {:?}", &rx_bytes[..bytes_read]);
            match Self::decode(&rx_bytes[..bytes_read]) {
                Ok(dcc_packet) => match &dcc_packet.body {
                    DccPacketBody::HeartBeat(_) => {
                        if let Some(s) = &self.sender {
                            s.try_send(ToFrontend::DccPacket(
                                self.addr.clone(),
                                dcc_packet.clone(),
                            ))
                            .unwrap();
                        }

                        let start = SystemTime::now();
                        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap().as_secs();

                        let mut reply = DccPacket {
                            header: dcc_packet.header.clone(),
                            body: DccPacketBody::HeartBeatRepl(since_the_epoch),
                        };
                        let buf = reply.encode();
                        eprint!("{:?}", buf);
                        writer.write_all(&buf).unwrap();
                        writer.flush().unwrap()
                    }

                    _ => {}
                },
                Err(_) => todo!(),
            }
        }
        Ok(())
    }
}

impl Dcc {
    fn decode(data: &[u8]) -> Result<DccPacket, &str> {
        match data {
            [body @ .., crc_h, crc_low]
                if body[0] == 0xfe
                    && data[1] == 0xfe
                    && MODBUS.checksum(&body) == ((*crc_h as u16) << 8) | *crc_low as u16 =>
            {
                let header = &body[2..26];
                let h: Header = header.try_into().expect("dcc header decode error");

                let b = match h.protocol {
                    x if x == Protocol::HeartBeat as u8 => {
                        let io_data: IoData = body[26..].try_into().unwrap();
                        eprintln!("{:?}",io_data);
                        DccPacketBody::HeartBeat(io_data)
                    }
                    _ => DccPacketBody::UnKnow(format!("{:?}",&body[26..])),
                };
                Ok(DccPacket {
                    header: h,
                    body: b,
                })
            }
            _ => Err("crc check error"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Header {
    pub version: u8,
    pub device_type: u8,
    pub device_id: u64,
    pub module: u8,
    pub module_id: u8,
    protocol: u8,
    sub_protocol: u8,
    pub msg_id: u64,
}

impl TryFrom<&[u8]> for Header {
    type Error = ();

    fn try_from(header: &[u8]) -> Result<Self, Self::Error> {
        Ok(Header {
            version: header[2],
            device_type: header[3],
            device_id: u64::from_be_bytes(header[4..12].try_into().unwrap()),
            module: header[12],
            module_id: header[13],
            protocol: header[14],
            sub_protocol: header[15],
            msg_id: u64::from_be_bytes(header[15..23].try_into().unwrap()),
        })
    }
}

enum Protocol {
    Boardcast = 0,
    HeartBeat = 1,
    Cmd = 2,
    Data = 3,
}

#[derive(Debug, Clone)]
pub enum DccCmd {
    DO(DO),
    AO(AO),
    VO(VO),
    T485,
}

#[derive(Debug, Clone)]
pub struct DccPacket {
    pub header: Header,
    pub body: DccPacketBody,
}

impl DccPacket {
    fn msg_id() -> u64 {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap().as_secs();
        since_the_epoch
    }

    fn encode(&mut self) -> Vec<u8> {
        let Self {
            header:
                Header {
                    version,
                    device_type,
                    device_id,
                    module,
                    module_id,
                    protocol,
                    sub_protocol,
                    msg_id,
                },
            body,
        } = self;

        let body_data = body.encode();
        let l = body_data.len() + 26;

        let mut buf = vec![];
        buf.put_u16(0xfefeu16);
        buf.put_u16(l as u16);
        buf.put_u8(*version);
        buf.put_u8(*device_type);
        buf.put_u64(*device_id);
        buf.put_u8(*module);
        buf.put_u8(*module_id);
        buf.put_u8(*protocol);
        buf.put_u8(*sub_protocol);
        buf.put_u64(*msg_id);
        buf.put(&body_data[..]);
        let crc = MODBUS.checksum(&buf);
        buf.put_u16(crc);
        buf
    }

    pub fn do_ctr(io: DO, device_id: &u64, version: &u8) -> Self {
        Self {
            header: Header {
                version: *version,
                device_type: 0,
                device_id: *device_id,
                module: IoType::DO as u8,
                module_id: io.index,
                protocol: Protocol::Cmd as u8,
                sub_protocol: 2u8,
                msg_id: Self::msg_id(),
            },
            body: DccPacketBody::DoCtr(io),
        }
    }

    pub fn ao_ctr(io: AO, device_id: &u64, version: &u8) -> Self {
        Self {
            header: Header {
                version: *version,
                device_type: 0,
                device_id: *device_id,
                module: IoType::AO as u8,
                module_id: io.index,
                protocol: Protocol::Cmd as u8,
                sub_protocol: 2u8,
                msg_id: Self::msg_id(),
            },
            body: DccPacketBody::AoCtr(io),
        }
    }

    pub fn vo_ctr(io: VO, device_id: &u64, version: &u8) -> Self {
        Self {
            header: Header {
                version: *version,
                device_type: 0,
                device_id: *device_id,
                module: IoType::VO as u8,
                module_id: io.index,
                protocol: Protocol::Cmd as u8,
                sub_protocol: 2u8,
                msg_id: Self::msg_id(),
            },
            body: DccPacketBody::VoCtr(io),
        }
    }
}

#[derive(Debug, Clone)]
pub enum DccPacketBody {
    HeartBeat(IoData),
    HeartBeatRepl(u64),
    DoCtr(DO),
    AoCtr(AO),
    VoCtr(VO),
    UnKnow(String),
}

impl DccPacketBody {
    fn encode(&self) -> Vec<u8> {
        let mut buf = vec![];
        match self {
            DccPacketBody::HeartBeat(_) => {}

            DccPacketBody::HeartBeatRepl(time) => {
                buf.put_u64(*time);
            }
            DccPacketBody::DoCtr(io) => {
           
                buf.put_u16(io.origin())
            }
            DccPacketBody::AoCtr(io) => {
           
                buf.put_u16(io.origin())
            }
            DccPacketBody::VoCtr(io) => {
              
                buf.put_u16(io.origin())
            }
            DccPacketBody::UnKnow(_) => {}
        }
        buf
    }
}


#[derive(Default, Debug, Clone)]
pub struct IoData {
    pub di_list: Vec<DI>,
    pub do_list: Vec<DO>,
    pub ai_list: Vec<AI>,
    pub ao_list: Vec<AO>,
    pub via_list: Vec<VIA>,
    pub vir_list: Vec<VIR>,
    pub vo_list: Vec<VO>,
    pub hardware_ver: u16,
    pub software_ver: u16,
}

impl TryFrom<&[u8]> for IoData {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let io_count = value[0];
        if value.len() < (io_count * 4 + 1).into() {
            return Err(());
        } else {
            let mut io_data = Self::default();
            for x in value[1..(io_count * 4 + 1).into()].chunks(4) {
                let io_type = x[0];
                let io_index = x[1];
                match io_type {
                    t if t == IoType::DI as u8 => io_data.di_list.push(DI::parse(x)),
                    t if t == IoType::DO as u8 => io_data.do_list.push(DO::parse(x)),
                    t if t == IoType::AI as u8 => io_data.ai_list.push(AI::parse(x)),
                    t if t == IoType::AO as u8 => io_data.ao_list.push(AO::parse(x)),
                    t if t == IoType::VI as u8 && io_index < 3 => {
                        io_data.via_list.push(VIA::parse(x))
                    }
                    t if t == IoType::VI as u8 => io_data.vir_list.push(VIR::parse(x)),
                    t if t == IoType::VO as u8 => io_data.vo_list.push(VO::parse(x)),
                    _ => eprintln!("unknow type"),
                }
            }

            Ok(io_data)
        }
    }
}

pub trait IO: Sized {
    fn parse(value: &[u8]) -> Self;

    fn origin(&self) -> u16;

    fn unit() -> &'static str {
        ""
    }
    fn encode() {}
    fn decode() {}
}


pub enum IoType {
    DI = 1,
    DO = 2,
    AI = 3,
    AO = 4,
    T485 = 5,
    VI = 7,
    VO = 8,
}

#[derive(Debug, Clone)]
pub struct DI {
    pub index: u8,
    pub origin_value: u16,
    pub parse_value: u16,
}

#[derive(Debug, Clone)]
pub struct DO {
    pub index: u8,
    pub origin_value: u16,
    pub parse_value: u16,
}
#[derive(Debug, Clone)]
pub struct AI {
    pub index: u8,
    pub origin_value: u16,
    pub parse_value: f32,
}
#[derive(Debug, Clone)]
pub struct AO {
    pub index: u8,
    pub origin_value: u16,
    pub parse_value: f32,
}
#[derive(Debug, Clone)]
pub struct VIA {
    pub index: u8,
    pub origin_value: u16,
    pub parse_value: f32,
}
#[derive(Debug, Clone)]
pub struct VIR {
    pub index: u8,
    pub origin_value: u16,
    pub parse_value: f32,
}
#[derive(Debug, Clone)]
pub struct VO {
    pub index: u8,
    pub origin_value: u16,
    pub parse_value: f32,
}

impl IO for DI {
    fn parse(value: &[u8]) -> Self {
        let v = ((value[2] as u16) << 8) | value[3] as u16;
        Self {
            index: value[1],
            origin_value: v,
            parse_value: v,
        }
    }

    fn origin(&self) -> u16 {
       self.parse_value
    }
}
impl IO for DO {
    fn parse(value: &[u8]) -> Self {
        let v = ((value[2] as u16) << 8) | value[3] as u16;
        Self {
            index: value[1],
            origin_value: v,
            parse_value: v,
        }
    }

    fn origin(&self) -> u16 {
        self.parse_value
    }
}

impl IO for AI {
    fn unit() -> &'static str {
        "mA"
    }
    fn parse(value: &[u8]) -> Self {
        let v = ((value[2] as u16) << 8) | value[3] as u16;
        Self {
            index: value[1],
            origin_value: v,
            parse_value: (v as f32 / 684.0),
        }
    }

    fn origin(&self) -> u16 {
     ( self.parse_value * 684.0) as u16
    }
}

impl IO for AO {
    fn unit() -> &'static str {
        "mA"
    }

    fn parse(value: &[u8]) -> Self {
        let v = ((value[2] as u16) << 8) | value[3] as u16;
        Self {
            index: value[1],
            origin_value: v,
            parse_value: v as f32 / 1000.0,
        }
    }

    fn origin(&self) -> u16 {
       (self.parse_value * 1000.0) as u16
    }
}

impl IO for VIA {
    fn unit() -> &'static str {
        "V"
    }

    fn parse(value: &[u8]) -> Self {
        let v = ((value[2] as u16) << 8) | value[3] as u16;
        Self {
            index: value[1],
            origin_value: v,
            parse_value: v as f32 / 1000.0,
        }
    }

    fn origin(&self) -> u16 {
        (self.parse_value * 1000.0) as u16
    }
}

impl IO for VIR {
    fn unit() -> &'static str {
        "Ω"
    }

    fn parse(value: &[u8]) -> Self {
        let v = ((value[2] as u16) << 8) | value[3] as u16;
        Self {
            index: value[1],
            origin_value: v,
            parse_value: 10000.0 * v as f32 / (3000.0 - v as f32),
        }
    }

    fn origin(&self) -> u16 {
        self.origin_value
    }
}

impl IO for VO {
    fn unit() -> &'static str {
        "V"
    }

    fn parse(value: &[u8]) -> Self {
        let v = ((value[2] as u16) << 8) | value[3] as u16;
        Self {
            index: value[1],
            origin_value: v,
            parse_value: v as f32 / 1000.0,
        }
    }

    fn origin(&self) -> u16 {
        (self.parse_value * 1000.0) as u16
    }
}

#[test]
fn heartbeat_decode_test() {
    let hb_pkt: &[u8] = &[
        254, 254, 0, 223, 1, 2, 0, 0, 68, 67, 67, 103, 45, 234, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 139,
        109, 49, 1, 0, 0, 0, 1, 1, 0, 0, 1, 2, 0, 0, 1, 3, 0, 0, 1, 4, 0, 0, 1, 5, 0, 0, 1, 6, 0,
        0, 1, 7, 0, 0, 1, 8, 0, 0, 1, 9, 0, 0, 1, 10, 0, 0, 1, 11, 0, 0, 1, 12, 0, 0, 1, 13, 0, 0,
        1, 14, 0, 0, 1, 15, 0, 0, 1, 16, 0, 0, 1, 17, 0, 0, 1, 18, 0, 0, 1, 19, 0, 0, 2, 0, 0, 0,
        2, 1, 0, 0, 2, 2, 0, 0, 2, 3, 0, 0, 2, 4, 0, 0, 3, 0, 0, 0, 3, 1, 0, 0, 3, 2, 0, 0, 3, 3,
        0, 0, 3, 4, 0, 0, 3, 5, 0, 0, 3, 6, 0, 0, 4, 0, 15, 160, 4, 1, 15, 160, 4, 2, 15, 160, 4,
        3, 15, 160, 4, 4, 15, 160, 7, 0, 0, 44, 7, 1, 0, 44, 7, 2, 0, 40, 7, 3, 11, 180, 7, 4, 11,
        179, 7, 5, 11, 180, 7, 6, 11, 179, 8, 0, 7, 189, 8, 1, 7, 189, 8, 2, 7, 189, 253, 0, 1, 13,
        254, 0, 1, 1, 215, 214,
    ];
    Dcc::decode(hb_pkt);
}
#[test]
fn heartbeat_repl_decode_test() {
  let d  =    &[254, 254, 0, 34, 1, 2, 0, 0, 68, 67, 67, 103, 45, 234, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 87, 0, 0, 0, 0, 100, 126, 147, 197, 47, 81];
 let pkt =  Dcc::decode(d).unwrap();
 eprintln!("{:?}",pkt.header)

}

#[test]
fn do_ctr_encode_test() {
  let d  =    &[254, 254, 0, 30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 2, 2, 0, 0, 0, 0, 100, 126, 149, 174, 2, 0, 0, 1, 255, 233];
 let pkt =  Dcc::decode(d).unwrap();
 eprintln!("{:?}",pkt)

}

