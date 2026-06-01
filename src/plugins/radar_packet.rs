use std::string::ToString;
use std::time::SystemTime;
use fastrand::u16;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use hdf5_metno::{File, Group, H5Type};
use num_complex::Complex;

/// Server binary version sourced from cargo at compile time.
pub static VERSION:&str = env!("CARGO_PKG_VERSION");

/// Network type for the packet, currently unused.
#[derive(Serialize,Deserialize,Debug)]
pub enum NetType {
    Server,
    Client,
    Archiver,
}

/// Identity info contains the network type and server version.
#[derive(Serialize,Deserialize,Debug)]
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
    pub(crate) rotation_speed:f64,
}

/// The radar data packet for use with dummy complex i16 data.
///
/// Contains an identity, time of recording, state, and the data vector.
#[derive(Serialize,Deserialize,Debug)]
pub struct ComPacket {
    pub(crate) identity:Identity,
    pub(crate) timestamp:f64,
    pub(crate) state:State,
    #[serde(with="serde_bytes")]
    pub(crate) data:Vec<u8>,
}

/// A struct to denote a change in the date/time archived playback
///
/// A SystemTime value denotes a date to search, while none requests live data
#[derive(Serialize,Deserialize,Debug)]
pub struct ArchivedPlayback {
    pub(crate) time:Option<SystemTime>
}

/// The radar packet for communication over the settings channel.
///
/// Contains an identity, optional archived time request, and optional settings state
#[skip_serializing_none]
#[derive(Serialize,Deserialize,Debug)]
pub struct ComPacketSettings {
    pub(crate) identity:Identity,
    pub(crate) playback:Option<ArchivedPlayback>,
    pub(crate) controls:Option<Vec<Setting>>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct Setting {
    name:String,
    min:Option<String>,
    max:Option<String>,
    step:Option<String>,
    unit:Option<String>,
    values:Option<Vec<Setting>>,
}

/// An Hdf5Object can be stored and retrieved from an HDF5 file.
pub trait Hdf5Object{
    /// Stores the object in the specified file.
    ///
    /// The object is stored in a group corresponding to its epoch time,
    /// and a subgroup corresponding to its subsecond millisecond time.
    fn to_hdf5(&self, file: &File) -> hdf5_metno::Result<()>;

    /// Retrieves an object from the specified file and group.
    ///
    /// Only retrieves an object from a group or fails, does not search for a specific object.
    fn from_hdf5() -> hdf5_metno::Result<Self> where Self: Sized;
}

pub trait ExportableSetting {
    fn get_default_settings(&self) -> Option<Vec<Setting>>;
    fn get_archive_settings(&self) -> Option<Vec<Setting>>;
}

impl ExportableSetting for State {
    fn get_default_settings(&self) -> Option<Vec<Setting>> {
        let mut vals = Vec::<Setting>::new();
        vals.push( Setting{
            name:"Samples".to_string(),
            min: Some("0".to_string()),
            max: Some(u16::MAX.to_string()),
            step: Some("1".to_string()),
            unit: None,
            values: None,
        });
        vals.push( Setting{
            name:"Rotation Rate".to_string(),
            min: Some("0.0".to_string()),
            max: Some("1.0".to_string()),
            step: None,
            unit: Some("Degrees/Sec".to_string()),
            values: None,
        });
        vals.push( Setting{
            name:"Sample Rate".to_string(),
            min: Some("0".to_string()),
            max: Some(u16::MAX.to_string()),
            step: None,
            unit: Some("Samples/Sec".to_string()),
            values: None,
        });
        Some(vals)
    }
    fn get_archive_settings(&self) -> Option<Vec<Setting>> {
        let mut vals = Vec::<Setting>::new();
        vals.push(Setting{
            name:"Playback Rate".to_string(),
            min: Some("0".to_string()),
            max: Some(u16::MAX.to_string()),
            step: None,
            unit: Some("Samples/Sec".to_string()),
            values: None,
        });
        Some(vals)
    }
}

/// Implementation of HDF5Object for ComPacketIntComplex.
impl Hdf5Object for ComPacket {
    fn to_hdf5(&self, file: &File) -> hdf5_metno::Result<()> {
        todo!()
    }

    fn from_hdf5() -> hdf5_metno::Result<Self> {
        todo!()
    }
}
