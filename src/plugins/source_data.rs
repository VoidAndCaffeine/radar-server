use std::time::{Duration, SystemTime, UNIX_EPOCH};
use bytemuck::bytes_of;
use num_complex::Complex;
use hdf5_metno::{File};
use crate::plugins::radar_packet::*;

/// The number of data samples per packet
///
/// ToDo: this value should be dynamic, and probably in the radar data packet
pub static NUM_SAMPLES:usize = 1000;

/// Organizational struct for DummyData.
pub struct DummyData;

#[derive(Copy, Clone)]
struct AntennaState {
    last_a: Option<f64>,
    last_t: Option<f64>,
    last_da_dt: Option<f64>,
    dt: f64,
}
impl AntennaState {
    pub fn new() -> AntennaState {
        AntennaState {
            last_a: None,
            last_t: None,
            last_da_dt: None,
            dt: 0.0,
        }
    }
    pub fn update(&mut self, angle: f64, time: f64) -> f64 {
        match self.last_a {
            None => {
                self.last_a = Some(angle);
                self.last_t = Some(time);
                self.dt = 0.0;
                5.0
            }
            Some(prev_a) if angle == prev_a => {
                if self.last_da_dt.is_some() {
                    self.dt = 0.0;
                    self.last_da_dt.unwrap()
                }else {
                    self.dt = 0.0;
                    5.0
                }
            }
            Some(prev_a) => {
                if let Some(prev_t) = self.last_t{
                    let dt = time - prev_t;
                    if dt > 0.0 {
                        let rate = (angle - prev_a) / dt;
                        self.last_t = Some(time);
                        self.last_a = Some(angle);
                        self.last_da_dt = Some(rate);
                        self.dt = dt;
                        rate
                    } else {
                        self.last_a = None;
                        self.last_t = None;
                        self.dt = 0.0;
                        5.0
                    }
                } else {
                    self.last_a = None;
                    self.last_t = None;
                    self.dt = 0.0;
                    5.0
                }
            }
        }
    }
}

pub struct DemoData{
    pub manual_delay: bool,
    pub delay:u16,
    idx:usize,
    antenna_state: [AntennaState; 4],
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
            manual_delay: false,
            delay: 0,
            idx:0,
            antenna_state: [AntennaState::new(),AntennaState::new(),AntennaState::new(),AntennaState::new()],
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
            byte_vec.extend_from_slice(bytes_of(&c));
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
        let antenna = self.antenna_ds[self.idx] as usize;

        self.state.rotation_rate = self.antenna_state[antenna].update(angle, timestamp);
        self.state.angle = angle;
        self.state.antenna = antenna as u8;
        self.state.enabled = self.enable_ds[self.idx] as u8 != 0;
        self.state.samples = samples;
        let mut data: Vec<u8> = Vec::with_capacity(samples as usize);

        for i in 0..samples as usize {
            let noise = 25;
            let c =
                Complex::new(self.real_ds[self.idx][i], self.imag_ds[self.idx][i]);
                    //+ Complex::new(fastrand::i32(-noise..=noise), fastrand::i32(-noise..=noise));
            data.extend_from_slice(bytes_of(&c));
        }

        if !self.manual_delay {self.delay = Duration::from_secs_f64(self.antenna_state[antenna].dt).as_millis() as u16;}

        self.idx = (self.idx + 1) % self.real_ds.len();
        ComPacket{identity,timestamp,state:self.state,data}
    }
}