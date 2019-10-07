use amethyst::core::Time;
use num_derive::FromPrimitive;
use shrinkwraprs::Shrinkwrap;
use std::sync::{
    atomic::{AtomicU32, AtomicU64, Ordering},
    Mutex,
};
use strum_macros::{AsRefStr, EnumIter};

const YEAR: u64 = 31_104_000; // 12 month in a year
const MONTH: u64 = 2_592_000; // 30 days in a month
const DAY: u64 = 86400; // 24 hours in a day
const HOUR: u64 = 3600; // 60 minutes in a hour
const MINUTE: u64 = 60;

pub mod scales {
    use super::*;

    const NORMAL: f32 = 10.0; // 10 minutes per day
    const FAST: f32 = 5.0; // 5 minutes per day
    const FASTEST: f32 = 2.5; // 2.5 minutes per day

    pub const fn normal() -> f32 {
        DAY as f32 / ((MINUTE as f32) * NORMAL)
    }

    pub const fn fast() -> f32 {
        DAY as f32 / ((MINUTE as f32) * FAST)
    }

    pub const fn fastest() -> f32 {
        DAY as f32 / ((MINUTE as f32) * FASTEST)
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    EnumIter,
    AsRefStr,
    FromPrimitive,
    serde::Serialize,
    serde::Deserialize,
)]
#[repr(u8)]
pub enum Day {
    Sunday = 0,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    EnumIter,
    AsRefStr,
    FromPrimitive,
    serde::Serialize,
    serde::Deserialize,
)]
#[repr(u8)]
pub enum Month {
    January = 1,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    EnumIter,
    AsRefStr,
    FromPrimitive,
    serde::Serialize,
    serde::Deserialize,
)]
#[repr(u8)]
pub enum Season {
    Spring = 0,
    Summer,
    Fall,
    Winter,
}

pub struct CalendarDate(Instant);
impl CalendarDate {
    pub fn day(&self) -> Day {
        use num_traits::FromPrimitive;
        Day::from_u64(7 - (self.0.day() % 7)).unwrap()
    }

    pub fn month(&self) -> Month {
        use num_traits::FromPrimitive;
        Month::from_u64(self.0.month()).unwrap()
    }

    pub fn year(&self) -> u64 {
        self.0.year()
    }

    pub fn day_of_month(&self) -> u64 {
        self.0.day()
    }
    pub fn day_of_week(&self) -> u64 {
        self.0.day() / 7
    }
    pub fn season(&self) -> Season {
        use num_traits::FromPrimitive;
        Season::from_u64(self.0.month() / 4).unwrap()
    }
}

// Experiment in interiorly mutable game time
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct WorldTime {
    pub epoch: AtomicU64,
    pub offset: AtomicU64,
    pub accumulator: AtomicU32,
    pub write_lock: Mutex<()>,
}
impl PartialEq for WorldTime {
    fn eq(&self, other: &Self) -> bool {
        self.epoch() == other.epoch() && self.offset() == other.offset()
    }
}

impl WorldTime {
    pub fn new(epoch: u64) -> Self {
        Self {
            epoch: AtomicU64::new(epoch),
            offset: AtomicU64::new(0),
            accumulator: AtomicU32::new(0),
            write_lock: Mutex::default(),
        }
    }

    pub fn accumulator(&self) -> f32 {
        f32::from_bits(self.accumulator.load(Ordering::Relaxed))
    }

    pub fn add_accumulator(&self, value: f32) {
        let _guard = self.write_lock.lock();

        self.accumulator
            .store((self.accumulator() + value).to_bits(), Ordering::Relaxed);
    }

    pub fn epoch(&self) -> u64 {
        self.epoch.load(Ordering::Relaxed)
    }

    pub fn offset(&self) -> u64 {
        self.offset.load(Ordering::Relaxed)
    }

    pub fn value(&self) -> u64 {
        self.epoch() + self.offset()
    }

    pub fn elapse(&self, time: &Time) {
        self.elapse_raw(time.delta_seconds());
    }

    pub fn elapse_raw(&self, delta: f32) {
        self.add_accumulator(delta);
        while self.accumulator() >= 1.0 {
            self.add_accumulator(-1.0);
            self.offset.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn now(&self) -> Instant {
        Instant(self.value())
    }
}

#[derive(
    Shrinkwrap,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Clone,
    Copy,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Instant(u64);
impl Instant {
    pub fn second(self) -> u64 {
        (self.0 % MINUTE)
    }

    pub fn minute(self) -> u64 {
        (self.0 % HOUR) / MINUTE
    }

    pub fn hour(self) -> u64 {
        (self.0 % DAY) / HOUR
    }

    pub fn day(self) -> u64 {
        (self.0 % MONTH) / DAY
    }

    pub fn month(self) -> u64 {
        (self.0 % YEAR) / MONTH
    }

    pub fn year(self) -> u64 {
        self.0 / YEAR
    }

    pub fn calendar(self) -> CalendarDate {
        CalendarDate(self)
    }

    pub fn value(self) -> u64 {
        self.0
    }
}
impl std::ops::Sub for Instant {
    type Output = Instant;

    fn sub(self, other: Instant) -> Instant {
        Instant(self.0 - other.0)
    }
}
impl std::ops::Add for Instant {
    type Output = Instant;

    fn add(self, other: Instant) -> Instant {
        Instant(self.0 + other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::init_test_log;

    #[test]
    fn clock_time() {
        init_test_log();

        let time = WorldTime::new(70347970);
        let now = time.now();
        log::trace!("epoch={}", time.epoch());
        assert_eq!(now.year(), 2);
        assert_eq!(now.month(), 3);
        assert_eq!(now.hour(), 5);
        assert_eq!(now.minute(), 6);
        assert_eq!(now.second(), 10);

        time.elapse_raw(10.0);
        let now = time.now();
        assert_eq!(now.year(), 2);
        assert_eq!(now.month(), 3);
        assert_eq!(now.day(), 4);
        assert_eq!(now.hour(), 5);
        assert_eq!(now.minute(), 6);
        assert_eq!(now.second(), 20);

        let cal = now.calendar();
        assert_eq!(cal.month(), Month::March);
        assert_eq!(cal.day(), Day::Wednesday);
        assert_eq!(cal.season(), Season::Spring);
    }
}
