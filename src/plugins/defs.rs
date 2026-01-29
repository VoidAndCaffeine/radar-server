pub static NUM_SAMPLES:usize = 1000;
pub trait DataSource {
    fn source() -> [f64; NUM_SAMPLES];
}