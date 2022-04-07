use crate::*;

#[near_bindgen]
impl Contract {
    pub fn get_percent(&self, ratio: Ratio) -> f64 {
        // TODO: Maybe it's better to return this result directly from from all `Ratio` getters
        ratio as f64 / 10000.0
    }

    pub fn min(f1: f64, f2: f64) -> f64 {
        if f1 < f2 {
            f1
        } else {
            f2
        }
    }

    pub fn max(f1: f64, f2: f64) -> f64 {
        if f1 > f2 {
            f1
        } else {
            f2
        }
    }
}
