pub mod entity;
pub mod noise;
pub mod rng;

pub struct CyclicCounter {
    curr: u32,
    min: u32,
    max: u32,
}

impl CyclicCounter {
    pub fn new(min: u32, max: u32) -> Self {
        Self {
            curr: min,
            min,
            max,
        }
    }

    pub fn cycle(&mut self) -> u32 {
        let c = self.curr;
        if self.curr == self.max {
            self.curr = self.min;
        } else {
            self.curr += 1;
        }
        c
    }
}
