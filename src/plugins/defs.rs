use std::string::ToString;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};



pub static NUM_SAMPLES:usize = 1000;
pub trait DataSource {
    fn source() -> [f64; NUM_SAMPLES];
}
pub static VERSION:&str = env!("CARGO_PKG_VERSION");

#[derive(Serialize,Deserialize,Debug)]
pub enum NetType {
    Server,
    Client,
    Archiver,
    Transformer,
}

#[derive(Serialize,Deserialize,Debug)]
pub struct Identity {
    pub(crate) net_type:NetType,
    pub(crate) version:String,
}
#[derive(Serialize,Deserialize,Debug)]
pub struct CryptInfo {
   //ToDo Cryptography stuff
}

#[derive(Serialize,Deserialize,Debug)]
pub struct Blanking {
    pub(crate) x:i64,
    pub(crate) y:i64,
}

#[derive(Serialize,Deserialize,Debug)]
pub struct State {
    pub(crate) range:i64,
    pub(crate) rotation_speed:f64,
    pub(crate) blanking: Blanking,
    pub(crate) attenuation:f64,
    pub(crate) tune:f64,
}


#[derive(Serialize,Deserialize,Debug)]
pub struct ComPacketFloat {
    pub(crate) id:Identity,
    pub(crate) time:SystemTime,
    pub(crate) state:State,
    pub(crate) data:Vec<f64>
}

#[derive(Serialize,Deserialize,Debug)]
pub struct ComPacketInt {
    pub(crate) id:Identity,
    pub(crate) time:SystemTime,
    pub(crate) state:State,
    pub(crate) data:Vec<i64>
}
