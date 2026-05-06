use std::time::SystemTime;
use num_complex::{Complex, ComplexDistribution};
use rand::distr::Distribution;
use hdf5_metno::File;
use rand_distr::{Normal, Uniform};

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

/// A struct that functions as a FloatDataSource.
pub trait FloatDataSource {
    /// Returns an array of f64s NUM_SAMPLES long
    fn source_normal_data(&mut self) -> [f64; NUM_SAMPLES];
}
pub trait ComplexDataSource {
    fn source_complex_data(&mut self) -> Vec<Complex<i16>>;
}

impl FloatDataSource for DummyData {
    fn source_normal_data(&mut self) -> [f64; NUM_SAMPLES] {
        let mut retarry = [0.0; NUM_SAMPLES];
        let mut rng = rand::rng();
        let normal = Normal::new(0.0, 1.0).expect("Invalid distribution");
        for i in 0..NUM_SAMPLES {
            retarry[i] = normal.sample(&mut rng);
        }
        retarry
    }
}

impl ComplexDataSource for DummyData {
    fn source_complex_data(&mut self) -> Vec<Complex<i16>> {
        let mut rng = rand::rng();
        let uniform = Uniform::new(i16::MIN, i16::MAX).expect("Invalid distribution");
        let mut ret: Vec<Complex<i16>> = Vec::with_capacity(NUM_SAMPLES);
        for _ in 0..NUM_SAMPLES {
            ret.push(Complex::new(uniform.sample(&mut rng), uniform.sample(&mut rng)));
        }
        ret
    }
}

impl ComplexDataSource for ArchivedData {
    fn source_complex_data(&mut self) -> Vec<Complex<i16>> {
        // check for new seek time
        // seek backwards only if new
        // find correct timestamp
        // set rate
        // set time
        todo!()
    }
}