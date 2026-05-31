use std::collections::btree_set::Iter;
use std::time::{SystemTime, UNIX_EPOCH};
use num_complex::{Complex, ComplexDistribution};
use rand::distr::Distribution;
use hdf5_metno::File;
use rand_distr::Uniform;
use crate::plugins::radar_packet::{Blanking, ComPacket, Identity, NetType, State, VERSION};

/// The number of data samples per packet
///
/// ToDo: this value should be dynamic, and probably in the radar data packet
pub static NUM_SAMPLES:usize = 1000;

/// Organizational struct for DummyData.
pub struct DummyData;

/// Organizational struct for DummyData.
pub struct ArchivedData{
    pub(crate) packet_time: SystemTime,
    pub(crate) time_next: Option<SystemTime>,
    pub(crate) rate: f64,
    pub(crate) file: File,
}

pub trait ComplexDataSource {
    fn source_complex_data(&mut self) -> ComPacket;
    fn get_state(&self) -> State{
        State{
            range: 0,
            rotation_speed: 0.0,
            blanking: Blanking{
                start_delay: 0.0,
                end_delay: 0.0,
                azimuth: 0.0,
                elevation: 0,
                region_id: 0,
            },
            attenuation:0.0,
            tune:0.0,
        }
    }
}

impl ComplexDataSource for DummyData {
    fn source_complex_data(&mut self) -> ComPacket {
        let mut rng = rand::rng();
        let uniform = Uniform::new(i32::MIN, i32::MAX).expect("Invalid distribution");
        let mut byte_vec: Vec<u8> = Vec::with_capacity(NUM_SAMPLES);
        for _ in 0..NUM_SAMPLES {
            let c = Complex::new(uniform.sample(&mut rng), uniform.sample(&mut rng));
            byte_vec.extend_from_slice(&c.re.to_le_bytes());
            byte_vec.extend_from_slice(&c.im.to_le_bytes());
        }

        ComPacket {
            identity: Identity{
                net_type: NetType::Server,
                version: VERSION.to_string(),
            },
            time: SystemTime::now().duration_since(UNIX_EPOCH)
                .expect("Time before EPOCH not supported")
                .as_secs_f64(),
            state: State{
                range: 0,
                rotation_speed: 0.0,
                blanking: Blanking{
                    start_delay: 0.0,
                    end_delay: 0.0,
                    azimuth: 0.0,
                    elevation: 0,
                    region_id: 0,
                },
                attenuation:0.0,
                tune:0.0,
            },
            data:byte_vec,
        }
    }
}

impl ComplexDataSource for ArchivedData {
    fn source_complex_data(&mut self) -> ComPacket {
        todo!()
    }
}