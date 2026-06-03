use std::io::repeat;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use num_complex::Complex;
use hdf5_metno::{File};
use crate::plugins::radar_packet::*;

/// The number of data samples per packet
///
/// ToDo: this value should be dynamic, and probably in the radar data packet
pub static NUM_SAMPLES:usize = 1000;

/// Organizational struct for DummyData.
pub struct DummyData;

pub struct DemoData{
    pub manual_delay: bool,
    pub delay:u16,
    idx:usize,
    idx_ln:usize,
    idx_a0:usize,
    idx_a1:usize,
    idx_a2:usize,
    idx_a3:usize,
    last_dt:f64,
    angle_ds: Vec<f64>,
    antenna_ds: Vec<f64>,
    real_ds:Vec<Vec<i32>>,
    imag_ds:Vec<Vec<i32>>,
    enable_ds:Vec<f64>,
    time_ds:Vec<f64>,
    state:State,
}

impl DemoData{
    pub fn new() -> DemoData{
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
        if angle_ds.is_empty() || antenna_ds.is_empty() || real_ds.is_empty() || imag_ds.is_empty() || enable_ds.is_empty() || time_ds.is_empty() {
            panic!("Invalid data length!");
        }
        DemoData{
            manual_delay: true,
            delay: 100,
            idx:0,
            idx_ln:0,
            idx_a0:0,
            idx_a1:32,
            idx_a2:64,
            idx_a3:96,
            last_dt:0.0,
            angle_ds,
            antenna_ds,
            real_ds,
            imag_ds,
            enable_ds,
            time_ds,
            state: State{
                angle: 0.0,
                antenna: 0,
                enabled: true,
                samples: NUM_SAMPLES as u64,
                rotation_rate: 0.0,
            }
        }
    }
    pub fn update_state(&mut self, conf: SettingData){
        if conf.playback_delay.is_some() {
            let pd = conf.playback_delay.unwrap();
            if pd != 0 {
                println!("Setting manual delay to {pd}");
                self.manual_delay = true;
                self.delay = pd;
            } else {
                println!("Setting manual delay to auto");
                self.manual_delay = false;
            }
        }
    }
}

pub trait ComplexDataSource {
    fn source_complex_data(&mut self) -> ComPacket;
    fn get_state(&self) -> State{
        State{
            angle: 0.0,
            antenna: 0,
            enabled: true,
            samples: NUM_SAMPLES as u64,
            rotation_rate: 0.0,
        }
    }
}

impl ComplexDataSource for DummyData {
    fn source_complex_data(&mut self) -> ComPacket {
        let mut byte_vec: Vec<u8> = Vec::with_capacity(NUM_SAMPLES);
        print!("nums:[");
        for _ in 0..NUM_SAMPLES {
            let c = Complex::new(fastrand::i32(i32::MIN..=i32::MAX),fastrand::i32(i32::MIN..=i32::MAX));
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
            state: self.get_state(),
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
        let angle = self.angle_ds[self.idx];
        let samples =self.real_ds[self.idx].len() as u64;

        self.state.rotation_rate = 5.0;
        self.state.angle = angle;
        self.state.antenna = self.antenna_ds[self.idx] as u8;
        self.state.enabled = self.enable_ds[self.idx] as u8 != 0;
        self.state.samples = samples;
        let mut data: Vec<u8> = Vec::with_capacity(samples as usize);

        let mut ii = 0;
        println!();
        while self.idx + ii < self.real_ds.len() && self.antenna_ds[self.idx + ii] == self.antenna_ds[self.idx] {
            for i in 0..samples as usize {
                let noise = 25;
                let c =
                    Complex::new(self.real_ds[self.idx][i], self.imag_ds[self.idx][i])
                        + Complex::new(fastrand::i32(-noise..=noise), fastrand::i32(-noise..=noise));
                data.extend_from_slice(&c.re.to_le_bytes());
                data.extend_from_slice(&c.im.to_le_bytes());
                print!("{}",c);
            }
            ii += 1;
        }
        println!();

        if !self.manual_delay {self.delay = Duration::from_secs_f64(0.16).as_millis() as u16;}
        let ant1 = self.antenna_ds[self.idx] as u8;
        if (ant1 == 0) {
            self.idx_a0 = self.idx;
        }
        if (ant1 == 1) {
            self.idx_a1 = self.idx;
        }
        if (ant1 == 2) {
            self.idx_a2 = self.idx;
        }
        if (ant1 == 3) {
            self.idx_a3 = self.idx;
        }
        let ant2 = self.antenna_ds[self.idx + ii] as u8;
        if (ant2 == 0) {
            self.idx_ln = self.idx_a0;
        }
        if (ant2 == 1) {
            self.idx_ln = self.idx_a1;
        }
        if (ant2 == 2) {
            self.idx_ln = self.idx_a2;
        }
        if (ant2 == 3) {
            self.idx_ln = self.idx_a3;
        }

        self.idx = (self.idx + ii) % self.real_ds.len();
        if self.idx == 0 {
            println!("repeat");
        }
        ComPacket{identity,timestamp,state:self.state,data}
    }
}