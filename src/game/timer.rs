use std::time::*;

#[derive(Clone, Debug)]
pub struct Timer {
    last_instant: Instant,
    start_instant: Instant,
    pub desired_period: Duration,
}

#[derive(Clone, Debug)]
pub struct Tick {
    pub last_instant: Instant,
    pub delta: Duration,
}

impl Timer {
    pub fn new(desired_period: Duration) -> Timer {
        let start_instant = Instant::now();
        Timer {
            desired_period,
            start_instant,
            last_instant: start_instant,
        }
    }

    pub fn last_instant(&self) -> Instant {
        self.last_instant
    }

    pub fn start_instant(&self) -> Instant {
        self.start_instant
    }

    pub fn dur_from_start(&self, instant: Instant) -> Duration {
        self.start_instant.duration_since(instant)
    }

    pub fn tick(&mut self) -> Tick {
        let last_instant = self.last_instant;
        self.last_instant = Instant::now();
        let delta = self.last_instant.duration_since(last_instant);
        Tick {
            last_instant,
            delta,
        }
    }
}

/// Converts a duration to a floating point number.
pub fn duration_as_f64(dur: Duration) -> f64 {
    let sec: u64 = dur.as_secs() * 1000;
    let milli: u64 = dur.subsec_millis() as u64;
    let result = (sec + milli) as f64 / 1000f64;
    result
}

#[test]
fn test_duration_as_f64() {
    let dur = Duration::from_secs(10);
    assert_eq!(duration_as_f64(dur), 10.0);
}
