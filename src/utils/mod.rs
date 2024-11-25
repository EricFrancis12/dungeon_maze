pub mod asset;
pub mod entity;
pub mod maze;
pub mod noise;
pub mod rng;

#[derive(Clone, Copy)]
pub struct IncrCounter {
    value: i32,
    incr: i32,
}

impl IncrCounter {
    pub fn new(value: i32, incr: i32) -> Self {
        if incr == 0 {
            panic!("expected non-zero incr");
        }
        Self { value, incr }
    }

    pub fn tick(&mut self) -> i32 {
        let v: i32 = self.value;
        self.value = if self.incr > 0 {
            _min(v + self.incr, 0)
        } else {
            _max(v + self.incr, 0)
        };
        v
    }
}

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

    pub fn value(&self) -> u32 {
        self.curr
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

pub fn _min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b {
        return a;
    }
    b
}

pub fn _max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b {
        return a;
    }
    b
}

pub fn _min_max_or_betw<T: PartialOrd>(min: T, max: T, betw: T) -> T {
    if min > betw {
        return min;
    }
    if max < betw {
        return max;
    }
    betw
}
