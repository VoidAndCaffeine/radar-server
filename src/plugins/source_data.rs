use crate::plugins::defs::DataSource;
use rand::distr::Distribution;
use rand_distr::Normal;

pub struct DummyNormalData;
impl DataSource for DummyNormalData {
    fn source() -> [f64; crate::plugins::defs::NUM_SAMPLES] {
        let mut retarry = [0.0; crate::plugins::defs::NUM_SAMPLES];
        let mut rng = rand::rng();
        let normal = Normal::new(0.0, 1.0).expect("Invalid distribution");
        for i in 0..crate::plugins::defs::NUM_SAMPLES {
            retarry[i] = normal.sample(&mut rng);
        }
        retarry
    }
}