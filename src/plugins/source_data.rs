use std::time::SystemTime;
use num_complex::{Complex, ComplexDistribution};
use rand::distr::Distribution;
use hdf5_metno::File;
use rand_distr::{Normal, Uniform};
use crate::plugins::radar_packet::{Blanking, ComPacketIntComplex, Identity, NetType, State, VERSION};

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
    fn source_complex_data(&mut self) -> ComPacketIntComplex;
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
    fn source_complex_data(&mut self) -> ComPacketIntComplex {
        let mut rng = rand::rng();
        let uniform = Uniform::new(i32::MIN, i32::MAX).expect("Invalid distribution");
        let mut data_vec: Vec<Complex<i32>> = Vec::with_capacity(NUM_SAMPLES);
        for _ in 0..NUM_SAMPLES {
            data_vec.push(Complex::new(uniform.sample(&mut rng), uniform.sample(&mut rng)));
        }
        ComPacketIntComplex {
            id: Identity{
                net_type: NetType::Server,
                version: VERSION.to_string(),
            },
            time: SystemTime::now(),
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
            data:data_vec,
        }
    }
}

impl ComplexDataSource for ArchivedData {
    fn source_complex_data(&mut self) -> ComPacketIntComplex {

        todo!()
    }
}