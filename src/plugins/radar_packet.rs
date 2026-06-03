use std::string::ToString;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use hdf5_metno::{File, H5Type};

/// Server binary version sourced from cargo at compile time.
pub static VERSION:&str = env!("CARGO_PKG_VERSION");

/// Network type for the packet, currently unused.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum NetType {
    Server,
    Client,
    Archiver,
}

#[derive(Serialize,Deserialize)]
#[serde(untagged)]
pub enum Settings {
    SettingsInfo(SettingsInfo),
    SettingsData(SettingsData),
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
    pub(crate) rotation_speed:f64,
}

/// The radar data packet for use with dummy complex i16 data.
///
/// Contains an identity, time of recording, state, and the data vector.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComPacket {
    pub(crate) identity:Identity,
    pub(crate) timestamp:f64,
    pub(crate) state:State,
    #[serde(with="serde_bytes")]
    pub(crate) data:Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerSetting {
    samples:u16,
    sample_rate:f64,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArchiverSetting {
    playback_delay:u16,
}

#[skip_serializing_none]
#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct SettingsData {
    pub(crate) identity:Identity,
    pub(crate) server_controls:Option<ServerSetting>,
    pub(crate) controls:Option<ArchiverSetting>,
}

/// The radar packet for communication over the settings channel.
///
/// Contains an identity, optional archived time request, and optional settings state
#[skip_serializing_none]
#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct SettingsInfo {
    pub(crate) identity:Identity,
    pub(crate) server_controls:Option<Vec<SettingData>>,
    pub(crate) controls:Option<Vec<SettingData>>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct SettingData {
    id: String,
    name:String,
    min:Option<String>,
    max:Option<String>,
    step:Option<String>,
    unit:Option<String>,
    values:Option<Vec<SettingData>>,
}

impl ArchiverSetting {
    pub fn get_archiver_settings() -> Option<Vec<SettingData>> {
        let mut vals = Vec::<SettingData>::new();
        vals.push(SettingData {
            id: "playback_delay".to_string(),
            name: "Playback Delay".to_string(),
            min: Some("0".to_string()),
            max: Some(u16::MAX.to_string()),
            step: Some("1".to_string()),
            unit: Some("Ms".to_string()),
            values: None,
        });
        Some(vals)
    }
}


impl ServerSetting {
    pub fn get_server_settings() -> Option<Vec<SettingData>> {
        let mut vals = Vec::<SettingData>::new();
        vals.push(SettingData {
            id: "samples".to_string(),
            name: "Samples".to_string(),
            min: Some("0".to_string()),
            max: Some(u16::MAX.to_string()),
            step: Some("1".to_string()),
            unit: None,
            values: None,
        });
        vals.push(SettingData {
            id: "sample_rate".to_string(),
            name: "Sample Rate".to_string(),
            min: Some("0".to_string()),
            max: Some(u16::MAX.to_string()),
            step: None,
            unit: Some("Samples/Sec".to_string()),
            values: None,
        });
        Some(vals)
    }
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

/// Implementation of HDF5Object for ComPacketIntComplex.
impl Hdf5Object for ComPacket {
    fn to_hdf5(&self, file: &File) -> hdf5_metno::Result<()> {
        todo!()
    }

    fn from_hdf5() -> hdf5_metno::Result<Self> {
        todo!()
    }
}
