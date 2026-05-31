use std::convert::identity;
use std::time::{SystemTime, UNIX_EPOCH};
use num_complex::Complex;
use rand::distr::Distribution;
use hdf5_metno::{Dataset, File};
use rand_distr::Uniform;
use crate::plugins::radar_packet::{Blanking, ComPacket, Identity, NetType, State, VERSION};

/// The number of data samples per packet
///
/// ToDo: this value should be dynamic, and probably in the radar data packet
pub static NUM_SAMPLES:usize = 1000;

/// Organizational struct for DummyData.
pub struct DummyData;

pub struct DemoData{
    idx:usize,
    angle_ds: Vec<f64>,
    antenna_ds: Vec<f64>,
    real_ds:Vec<Vec<i32>>,
    imag_ds:Vec<Vec<i32>>,
    enable_ds:Vec<f64>,
    time_ds:Vec<f64>,
}

impl DemoData{
    pub fn new() -> DemoData{
        let idx = 0;
        let file = File::open("demo/20260519_dabob_first.hdf5").expect("Failed to open demo file!");
        let angle_ds:Vec<f64> = file.dataset("angle").expect("Failed to open angle dataset")
            .read_1d::<f64>().expect("Failed to read angle dataset").to_vec();
        let antenna_ds:Vec<f64> = file.dataset("antenna").expect("Failed to open antenna data")
            .read_1d::<f64>().expect("Failed to read antenna dataset").to_vec();
        let real_ds:Vec<Vec<i32>> = file.dataset("data_r").expect("Failed to open real data")
            .read_2d::<i32>().expect("failed to read real dataset")
            .rows().into_iter().map(|row| row.to_vec()).collect();
        let imag_ds:Vec<Vec<i32>> = file.dataset("data_i").expect("Failed to open imaginary data")
            .read_2d::<i32>().expect("failed to read imaginary dataset")
            .rows().into_iter().map(|row| row.to_vec()).collect();
        let enable_ds:Vec<f64> = file.dataset("enable").expect("Failed to open enable data")
            .read_1d::<f64>().expect("failed to read enable data").to_vec();
        let time_ds :Vec<f64>= file.dataset("time").expect("Failed to open time data")
            .read_1d::<f64>().expect("failed to read time data").to_vec();
        DemoData{
            idx,
            angle_ds,
            antenna_ds,
            real_ds,
            imag_ds,
            enable_ds,
            time_ds
        }
    }
}

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
            angle: 0.0,
            antenna: 0,
            enabled: true,
            samples: NUM_SAMPLES as u64,
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
        print!("nums:[");
        for _ in 0..NUM_SAMPLES {
            let c = Complex::new(uniform.sample(&mut rng), uniform.sample(&mut rng));
            print!("({}, {}), ",c.re,c.im);
            byte_vec.extend_from_slice(&c.re.to_le_bytes());
            byte_vec.extend_from_slice(&c.im.to_le_bytes());
        }
        print!("]\n");

        ComPacket {
            identity: Identity{
                net_type: NetType::Server,
                version: VERSION.to_string(),
            },
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)
                .expect("Time before EPOCH not supported")
                .as_secs_f64(),
            state: State{
                angle: 0.0,
                antenna: 0,
                enabled: true,
                samples: NUM_SAMPLES as u64,
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

impl ComplexDataSource for DemoData {
    fn source_complex_data(&mut self) -> ComPacket {
        let identity = Identity{
            net_type: NetType::Server,
            version: VERSION.to_string(),
        };
        let timestamp = self.time_ds[self.idx];

        let state: State = State{
            angle:self.angle_ds[self.idx],
            antenna: self.antenna_ds[self.idx] as u8,
            enabled: self.enable_ds[self.idx] as u8 != 0,
            samples: self.real_ds[self.idx].len() as u64,
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
        };
        let mut data: Vec<u8> = Vec::with_capacity(NUM_SAMPLES);
        for i in 0..NUM_SAMPLES {
            let c = Complex::new(self.real_ds[self.idx][i], self.imag_ds[self.idx][i]);
            data.extend_from_slice(&c.re.to_le_bytes());
            data.extend_from_slice(&c.im.to_le_bytes());
        }
        self.idx += 1;
        ComPacket{identity,timestamp,state,data}
    }
}