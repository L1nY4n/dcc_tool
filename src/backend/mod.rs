use std::{
    collections::HashMap,
    net::TcpListener,
    sync::{Arc, Mutex},
};

use crossbeam::{
    channel::{unbounded, Receiver, Sender},
    select,
};
pub mod message;

use message::{ToBackend, ToFrontend};
use tracing::error;

use crate::model::{Dcc, DccCmd};

pub struct Backend {
    dccs: Vec<Dcc>,
    back_tx: Sender<ToFrontend>,
    back_rx: Receiver<ToBackend>,
    dcc_pipe: Arc<Mutex<HashMap<String, Sender<DccCmd>>>>,
}

impl Backend {
    pub fn new(back_tx: Sender<ToFrontend>, back_rx: Receiver<ToBackend>, codes: String) -> Self {
        Self {
            back_tx,
            back_rx,
            dccs: vec![],
            dcc_pipe: Arc::new(Mutex::new(HashMap::default())),
        }
    }

    pub fn init(self) {
        let listener = TcpListener::bind("0.0.0.0:6000").unwrap();
        let dcc_pipe_clone = Arc::clone(&self.dcc_pipe);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                if let Ok(client) = stream {
                 
                    let (dcc_s, dcc_r) = unbounded();
                    let addr = client.peer_addr().unwrap().ip().to_string();
                    eprintln!("income {}",addr);
                    let mut new_dcc = Dcc::default();
                    new_dcc.addr = addr.clone();
                    {
                        let mut dcc_pipe = dcc_pipe_clone.lock().unwrap();
                        dcc_pipe.insert(addr, dcc_s);
                    }
                    let tx_clone =  self.back_tx.clone();
                    std::thread::spawn( move || {
                    new_dcc.init(client, tx_clone, dcc_r);
                });
                }
            }
        });
        loop {
            select! {
                recv(self.back_rx)->msg =>{
                     match msg {
                         Ok(m)=>{
                                match m {
                                    ToBackend::Refresh=>{
                                      //  self.refetch(false);
                                    },
                                    ToBackend::SetInterval(interval) => {
                                        eprintln!("{:?}",interval);
                                        },
                                    ToBackend::StockAdd(code) => {
                                      let dcc = &self.dccs[0];
                                     dcc.send();

                                    },
                                  ToBackend::StockDel(code) => {

                                    },
                                    ToBackend::DccCmd(addr,cmd) =>{
                                        {
                                            let  dcc_pipe = &self.dcc_pipe.lock().unwrap();
                                            if let Some(s) = dcc_pipe.get(&addr){
                                             match    s.try_send(cmd) {
                                                Ok(_) => { eprintln!("send success")
                                            },
                                                Err(err) =>{eprintln!("{}",err)},
                                                }
                                          }
                                        }
                                    },

                                     }
                                        }
                         Err(e) => {
                                error!("receive ToBackend msg faild : {}",e)
                          }
                        }
                    },
            }
        }
    }
}
