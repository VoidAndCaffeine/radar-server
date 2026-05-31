use std::string::ToString;
use std::time::SystemTime;
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

/// The blanking region, current values are placeholders
///
/// ToDo: get proper values
#[derive(H5Type,Clone,Copy,Serialize,Deserialize,Debug)]
#[repr(C)]
pub struct Blanking {
    pub(crate) start_delay:f32,
    pub(crate) end_delay:f32,
    pub(crate) azimuth:f32,
    pub(crate) elevation:i32,
    pub(crate) region_id:i32,
}

/// The state of the radar at the time of recording.
///
/// ToDo: NUM_SAMPLES should be stored here probably.
#[derive(H5Type,Clone,Copy,Serialize,Deserialize,Debug)]
#[repr(C)]
pub struct State {
    pub(crate) range:i64,
    pub(crate) rotation_speed:f64,
    pub(crate) blanking: Blanking,
    pub(crate) attenuation:f64,
    pub(crate) tune:f64,
}

/// The radar data packet for use with dummy complex i16 data.
///
/// Contains an identity, time of recording, state, and the data vector.
#[derive(Serialize,Deserialize,Debug)]
pub struct ComPacket {
    pub(crate) identity:Identity,
    pub(crate) time:f64,
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
#[derive(Serialize,Deserialize,Debug)]
pub struct ComPacketSettings {
    pub(crate) identity:Identity,
    pub(crate) first_time:bool,
    pub(crate) playback:Option<ArchivedPlayback>,
    pub(crate) setting:Option<Setting>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct Setting {
    name:String,
    min:Option<i32>,
    max:Option<i32>,
    step:Option<i32>,
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

pub trait ExportbleSetting {
    fn get_setting(&self) -> Setting;
}

impl ExportbleSetting for State {
    fn  get_setting(&self) -> Setting {
        let mut vals = Vec::<Setting>::new();
        vals.push( Setting{
            name:"Range".to_string(),
            min: Option::from(0),
            max: Option::from(0xfffffff),
            step: None,
            values: None,
        });
        vals.push( Setting{
            name:"Rotation Rate".to_string(),
            min: Option::from(0),
            max: Option::from(0xfffffff),
            step: None,
            values: None,
        });
        vals.push(self.blanking.get_setting());
        vals.push( Setting{
            name:"Attenuation".to_string(),
            min: Option::from(0),
            max: Option::from(0xfffffff),
            step: None,
            values: None,
        });
        vals.push( Setting{
            name:"Tune".to_string(),
            min: Option::from(0),
            max: Option::from(0xfffffff),
            step: None,
            values: None,
        });

        Setting{
            name: "Radar State".to_string(),
            min: None,
            max: None,
            step: None,
            values: Option::from(vals),
        }
    }
}

impl ExportbleSetting for Blanking {
    fn  get_setting(&self) -> Setting {
        let mut vals = Vec::<Setting>::new();
        vals.push( Setting{
            name:"Start Delay".to_string(),
            min: Option::from(0),
            max: Option::from(0xfffffff),
            step: None,
            values: None,
        });
        vals.push( Setting{
            name:"End Delay".to_string(),
            min: Option::from(0),
            max: Option::from(0xfffffff),
            step: None,
            values: None,
        });
        vals.push( Setting{
            name:"Azimuth".to_string(),
            min: Option::from(0),
            max: Option::from(0xfffffff),
            step: None,
            values: None,
        });
        vals.push( Setting{
            name:"Elevation".to_string(),
            min: Option::from(0),
            max: Option::from(0xfffffff),
            step: None,
            values: None,
        });
        vals.push( Setting{
            name:"Region ID".to_string(),
            min: Option::from(0),
            max: Option::from(0xfffffff),
            step: Option::from(1),
            values: None,
        });

        Setting{
            name: "Blanking Region".to_string(),
            min: None,
            max: None,
            step: None,
            values: Option::from(vals),
        }
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
