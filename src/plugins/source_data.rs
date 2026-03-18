use rand::distr::Distribution;
use rand_distr::Normal;

/// The number of data samples per packet
///
/// ToDo: this value should be dynamic, and probably in the radar data packet
pub static NUM_SAMPLES:usize = 1000;
/// Organizational struct for DummyData.
pub struct DummyNormalData;
/// A struct that functions as a FloatDataSource.
pub trait FloatDataSource {
    /// Returns an array of f64s NUM_SAMPLES long
    fn source() -> [f64; NUM_SAMPLES];
}
impl FloatDataSource for DummyNormalData {
    fn source() -> [f64; NUM_SAMPLES] {
        let mut retarry = [0.0; NUM_SAMPLES];
        let mut rng = rand::rng();
        let normal = Normal::new(0.0, 1.0).expect("Invalid distribution");
        for i in 0..NUM_SAMPLES {
            retarry[i] = normal.sample(&mut rng);
        }
        retarry
    }
}