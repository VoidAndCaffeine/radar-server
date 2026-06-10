use std::string::ToString;
use bytemuck::{cast_slice, pod_collect_to_vec};
use serde::{Deserialize, Serialize};
use serde_with::{skip_serializing_none};
use hdf5_metno::{Extent, File, H5Type};
use num_complex::Complex;
use crate::plugins::radar_packet::NetType::Archiver;

/// Server binary version sourced from cargo at compile time.
pub static VERSION:&str = env!("CARGO_PKG_VERSION");

/// Network type for the packet, currently unused.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum NetType {
    Server,
    Client,
    Archiver,
}

#[derive(Serialize,Deserialize,Debug, PartialEq,Clone)]
#[serde(untagged)]
pub enum SettingType{
    SettingData(SettingData),
    SettingInfo(Vec<SettingInfo>),
}
/// Identity info contains the network type and server version.
#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct Identity {
    pub(crate) net_type:NetType,
    pub(crate) version:String,
}

/// The state of the radar at the time of recording.
///
/// ToDo: NUM_SAMPLES should be stored here probably.
#[derive(H5Type,Clone,Copy,Serialize,Deserialize,Debug)]
#[repr(C)]
pub struct State {
    pub(crate) angle:f64,
    pub(crate) antenna:u8,
    pub(crate) enabled:bool,
    pub(crate) samples:u64,
    pub(crate) rotation_rate:f64,
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SettingData {
    pub samples:Option<u16>,
    pub sample_rate:Option<f64>,
    pub playback_delay:Option<u16>,
}


#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct SettingsPacket {
    pub(crate) identity:Identity,
    pub(crate) controls:SettingType,
}
/// The radar data packet for use with dummy complex i16 data.
///
/// Contains an identity, time of recording, state, and the data vector.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComPacket {
    pub(crate) identity:Identity,
    pub(crate) timestamp:f64,
    pub(crate) state:State,
    #[serde(with = "serde_bytes")]
    pub(crate) data:Vec<u8>,
}

#[derive(H5Type,Serialize,Clone,Copy)]
#[repr(C)]
struct HDF5Packet {
    timestamp:f64,
    state:State,
}

impl From<&ComPacket> for HDF5Packet {
    fn from(packet: &ComPacket) -> Self {
        Self {
            timestamp: packet.timestamp,
            state: packet.state,
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SettingInfo {
    id: String,
    name:String,
    min:Option<String>,
    max:Option<String>,
    step:Option<String>,
    unit:Option<String>,
    values:Option<Vec<SettingInfo>>,
}
impl SettingInfo {
    pub fn get_archiver_settings() -> Vec<SettingInfo> {
        let mut vals = Vec::<SettingInfo>::new();
        vals.push(SettingInfo {
            id: "playback_delay".to_string(),
            name: "Playback Delay".to_string(),
            min: Some("0".to_string()),
            max: Some(u16::MAX.to_string()),
            step: Some("1".to_string()),
            unit: Some("Ms".to_string()),
            values: None,
        });
        vals
    }
    pub fn get_server_settings() -> Vec<SettingInfo> {
        let mut vals = Vec::<SettingInfo>::new();
        vals.push(SettingInfo {
            id: "samples".to_string(),
            name: "Samples".to_string(),
            min: Some("0".to_string()),
            max: Some(u16::MAX.to_string()),
            step: Some("1".to_string()),
            unit: None,
            values: None,
        });
        vals.push(SettingInfo {
            id: "sample_rate".to_string(),
            name: "Sample Rate".to_string(),
            min: Some("0".to_string()),
            max: Some(u16::MAX.to_string()),
            step: None,
            unit: Some("Samples/Sec".to_string()),
            values: None,
        });
        vals
    }
}

/// An Hdf5Object can be stored and retrieved from an HDF5 file.
pub trait Hdf5Object{
    /// Stores the object in the specified file.
    ///
    /// The object is stored in a group corresponding to its epoch time,
    /// and a subgroup corresponding to its subsecond millisecond time.
    fn to_hdf5(&self, file:&mut File) -> hdf5_metno::Result<()>;

    /// Retrieves an object from the specified file and group.
    ///
    /// Only retrieves an object from a group or fails, does not search for a specific object.
    fn from_hdf5(idx:usize, file: &File) -> hdf5_metno::Result<Self> where Self: Sized;
}

/// Implementation of HDF5Object for ComPacketIntComplex.
impl Hdf5Object for ComPacket {
    fn to_hdf5(&self, file: &mut File) -> hdf5_metno::Result<()> {
        let timestamps = match file.dataset("timestamps") {
            Ok(ds) => ds,
            Err(_) => {
                println!("creating new timestamps dataset");
                file.new_dataset::<f64>()
                    .chunk((1,))
                    .shape(Extent::resizable(0))
                    .create("timestamps").expect("failed to create new dataset for timestamps")
            }
        };

        let metadata = match file.dataset("metadata") {
            Ok(ds) => ds,
            Err(_) => {
                println!("creating new metadata dataset");
                file.new_dataset::<HDF5Packet>()
                    .chunk((1,))
                    .shape(Extent::resizable(0))
                    .create("metadata").expect("failed to create new dataset for timestamps")
            }
        };

        let data = match file.dataset("data") {
            Ok(ds) => ds,
            Err(_) => {
                println!("creating new data dataset");
                file.new_dataset::<Complex<i32>>()
                    .chunk((1,2048))
                    .shape((1..,2048))
                    .create("data").expect("failed to create new dataset for timestamps")
            }
        };

        let ds:&[Complex<i32>] = cast_slice(self.data.as_slice());

        let idx = timestamps.size();
        timestamps.resize(idx + 1)?;
        metadata.resize(idx + 1)?;
        data.resize((idx+1,2048))?;
        timestamps.write_slice(&[self.timestamp],idx..idx+1)?;
        metadata.write_slice(&[HDF5Packet::from(self)],idx..idx+1)?;
        data.write_slice(ds, (idx,..))?;
        file.flush()?;
        Ok(())
    }

    fn from_hdf5(idx:usize,file: &File) -> hdf5_metno::Result<Self> {
        let metadata = match file.dataset("metadata") {
            Ok(ds) => ds,
            Err(e) => return Err(e),
        };

        let data = match file.dataset("data") {
            Ok(ds) => ds,
            Err(e) => return Err(e)
        };
        let meta:HDF5Packet = metadata.read_slice((idx..)).expect("Failed to read metadata").to_vec()[0];
        let data:Vec<Complex<i32>> = data.read_slice((idx,..)).expect("failed to read data").to_vec();

        Ok(ComPacket{
            identity:Identity{
                net_type:Archiver,
                version: VERSION.to_string()
            },
            timestamp: meta.timestamp,
            state:meta.state,
            data:pod_collect_to_vec(data.as_slice())  // ToDo: i think this involves a clone, there has to be a better way
        })
    }
}
