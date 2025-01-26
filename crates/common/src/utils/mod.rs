pub mod debug;
pub mod entity;
pub mod maze;
pub mod noise;
pub mod rng;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IncrCounter {
    value: i32,
    incr: i32,
}

impl IncrCounter {
    pub fn new(value: i32, incr: i32) -> Self {
        assert_ne!(incr, 0, "expected non-zero incr");
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

    pub fn get_value(&self) -> i32 {
        self.value
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

#[cfg(debug_assertions)]
pub fn contains_any<T, S1, S2>(sized_1: S1, sized_2: S2) -> bool
where
    T: PartialEq,
    S1: IntoIterator<Item = T>,
    S2: IntoIterator<Item = T>,
    S2: Clone,
{
    let sized_2_vec: Vec<T> = sized_2.into_iter().collect();
    for t in sized_1 {
        if sized_2_vec.iter().any(|_t| _t == &t) {
            return true;
        }
    }
    false
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

pub fn must_find_one<I, P>(iterable: I, predicate: P) -> I::Item
where
    I: IntoIterator,
    P: FnMut(&I::Item) -> bool,
{
    let items: Vec<I::Item> = iterable.into_iter().filter(predicate).collect();
    assert_eq!(items.len(), 1);
    items.into_iter().next().unwrap()
}
