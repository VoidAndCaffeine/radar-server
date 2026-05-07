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
    ///
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
//
// /// The radar data packet for use with dummy float data.
// ///
// /// Contains an identity, time of recording, state, and the data vector.
// #[derive(Serialize,Deserialize,Debug)]
// pub struct ComPacketFloat {
//     pub(crate) id:Identity,
//     pub(crate) time:SystemTime,
//     pub(crate) state:State,
//     pub(crate) data:Vec<f64>
// }
//
/// The radar data packet for use with dummy complex i16 data.
///
/// Contains an identity, time of recording, state, and the data vector.
#[derive(Serialize,Deserialize,Debug)]
pub struct ComPacketIntComplex {
    pub(crate) id:Identity,
    pub(crate) time:SystemTime,
    pub(crate) state:State,
    pub(crate) data:Vec<Complex<i16>>
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
    pub(crate) id:Identity,
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
    fn from_hdf5(group: &Group, time: SystemTime) -> hdf5_metno::Result<Self> where Self: Sized;
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
//
// /// Implementation of HDF5Object for ComPacketFloat.
// impl Hdf5Object for ComPacketFloat {
//     fn to_hdf5(&self, file: &File) -> hdf5_metno::Result<()> {
//         let packet_time = self.time.duration_since(SystemTime::UNIX_EPOCH)
//             .expect("Time went backwards");
//         let packet_secs = packet_time.as_secs().to_string();
//         let packet_ns = packet_time.subsec_nanos().to_string();
//         println!("Packet Time: {}\nPacket NSecs: {}", packet_secs, packet_ns);
//
//         let super_group = file.create_group(packet_secs.as_str()).expect("Unable to create group");
//         let group = super_group.create_group(packet_ns.as_str())?;
//
//         let s = serde_json::to_string(&self.id).expect("Unable to serialize to json");
//         let attr = group.new_attr::<u8>().shape(s.len()).create("Identity")?;
//         attr.write(s.as_bytes())?;
//         println!("Wrote Identity");
//         let attr = group.new_attr::<State>().create("State")?;
//         attr.write_scalar(&self.state)?;
//         println!("Wrote State");
//
//         let data_ds = group.new_dataset::<f64>()
//             .shape(self.data.len())
//             .create("Data")?;
//         println!("Wrote Data");
//         data_ds.write(&self.data)?;
//         file.flush()?;
//         Ok(())
//     }
//
//     fn from_hdf5(group: &Group, time:SystemTime) -> hdf5_metno::Result<Self> {
//         let id_attr = group.attr("Identity")?;
//         let id_bytes = id_attr.read_raw::<u8>()?;
//         let id_json = String::from_utf8(id_bytes).unwrap();
//         let id = serde_json::from_str(id_json.as_str()).expect("Unable to deserialize from json");
//
//         let state_attr = group.attr("State")?;
//         let state = state_attr.read_scalar()?;
//
//         let data_ds = group.dataset("Data")?;
//         let data = data_ds.read_raw::<f64>()?;
//
//         Ok(ComPacketFloat{id,time,state,data})
//     }
// }

/// Implementation of HDF5Object for ComPacketIntComplex.
impl Hdf5Object for ComPacketIntComplex {
    fn to_hdf5(&self, file: &File) -> hdf5_metno::Result<()> {
        let packet_time = self.time.duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards");
        let packet_secs = packet_time.as_secs().to_string();
        let packet_ns = packet_time.subsec_nanos().to_string();
        println!("Packet Time: {}\nPacket NSecs: {}", packet_secs, packet_ns);

        let super_group = file.create_group(packet_secs.as_str()).expect("Unable to create group");
        let group = super_group.create_group(packet_ns.as_str())?;

        let s = serde_json::to_string(&self.id).expect("Unable to serialize to json");
        let attr = group.new_attr::<u8>().shape(s.len()).create("Identity")?;
        attr.write(s.as_bytes())?;
        println!("Wrote Identity");
        let attr = group.new_attr::<State>().create("State")?;
        attr.write_scalar(&self.state)?;
        println!("Wrote State");

        let data_ds = group.new_dataset::<Complex<i16>>()
            .shape(self.data.len())
            .create("Data")?;
        println!("Wrote Data");
        data_ds.write(&self.data)?;
        file.flush()?;
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
        let data = data_ds.read_raw::<Complex<i16>>()?;

        Ok(ComPacketIntComplex{id,time,state,data})
    }
}
