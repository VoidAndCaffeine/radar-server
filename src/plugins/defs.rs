use std::any::Any;
use std::string::ToString;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use hdf5_metno::{File, Group, H5Type, Result};
use chrono::{DateTime, Utc};
use log::error;

pub static NUM_SAMPLES:usize = 1000;
pub trait DataSource {
    fn source() -> [f64; NUM_SAMPLES];
}

pub trait Hdf5Object{
    fn to_hdf5(&self, file: &File) -> hdf5_metno::Result<()>;
    fn from_hdf5(group: &Group, time: SystemTime) -> hdf5_metno::Result<Self> where Self: Sized;
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

#[derive(H5Type,Clone,Copy,Serialize,Deserialize,Debug)]
#[repr(C)]
pub struct Blanking {
    pub(crate) x:i64,
    pub(crate) y:i64,
}

#[derive(H5Type,Clone,Copy,Serialize,Deserialize,Debug)]
#[repr(C)]
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
//
// #[derive(Serialize,Deserialize,Debug,)]
// pub struct ComPacketInt {
//     pub(crate) id:Identity,
//     pub(crate) time:SystemTime,
//     pub(crate) state:State,
//     pub(crate) data:Vec<i64>
// }
//
impl Hdf5Object for ComPacketFloat {
    fn to_hdf5(&self, file: &File) -> hdf5_metno::Result<()> {
        let packet_time = self.time.duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards");
        let packet_secs = packet_time.as_secs().to_string();
        let packet_ns = packet_time.subsec_nanos().to_string();

        let super_group = file.create_group(packet_secs.as_str())?;
        let group = super_group.create_group(packet_ns.as_str())?;

        let attr = group.new_attr::<u8>().create("Identity")?;
        let s = serde_json::to_string(&self.id).expect("Unable to serialize to json");
        attr.write(s.as_bytes())?;
        let attr = group.new_attr::<State>().create("State")?;
        attr.write_scalar(&self.state)?;

        let data_ds = group.new_dataset::<f64>()
            .shape(self.data.len())
            .create("Data")?;
        data_ds.write(&self.data)?;

        Ok(())
    }
    
    fn from_hdf5(group: &Group, time:SystemTime) -> hdf5_metno::Result<Self> {
        let id_attr = group.attr("Identity")?;
        let id_bytes = id_attr.read_raw::<u8>()?;
        let id_json = String::from_utf8(id_bytes).unwrap();
        let id = serde_json::from_str(id_json.as_str()).expect("Unable to deserialize from json");

        let state_attr = group.attr("State")?;
        let state = state_attr.read_scalar()?;

        let data_ds = group.dataset("Data")?;
        let data = data_ds.read_raw::<f64>()?;

        Ok(ComPacketFloat{id,time,state,data})
    }
}
