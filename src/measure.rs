//! Repeated timing with warm-up, and allocation accounting.
//!
//! **Timing.** Wall clock (`std::time::Instant`) on a single thread:
//! `warmup` runs are discarded, then `reps` runs are measured. Reported:
//! median, quartiles, minimum, maximum and the median absolute deviation.
//!
//! **Memory.** [`Counting`] is a global allocator wrapper. It is installed
//! only by the benchmark binary, so in tests every reading is 0. It counts
//! the bytes allocated and the peak of live bytes above the start of the
//! measured closure.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::time::Instant;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Timing {
    pub warmup: usize,
    pub reps: usize,
    pub median_us: f64,
    pub p25_us: f64,
    pub p75_us: f64,
    pub min_us: f64,
    pub max_us: f64,
    pub mad_us: f64,
}

fn quantile(sorted: &[f64], q: f64) -> f64 {
    let pos = q * (sorted.len() - 1) as f64;
    let (lo, hi) = (pos.floor() as usize, pos.ceil() as usize);
    sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo as f64)
}

pub fn time(warmup: usize, reps: usize, mut f: impl FnMut()) -> Timing {
    for _ in 0..warmup { f(); }
    let mut t: Vec<f64> = (0..reps).map(|_| { let s = Instant::now(); f(); s.elapsed().as_secs_f64() * 1e6 }).collect();
    t.sort_by(|a, b| a.total_cmp(b));
    let median = quantile(&t, 0.5);
    let mut dev: Vec<f64> = t.iter().map(|x| (x - median).abs()).collect();
    dev.sort_by(|a, b| a.total_cmp(b));
    Timing { warmup, reps, median_us: median, p25_us: quantile(&t, 0.25), p75_us: quantile(&t, 0.75),
             min_us: t[0], max_us: t[reps - 1], mad_us: quantile(&dev, 0.5) }
}

pub struct Counting;

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static TOTAL: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(l) };
        if !p.is_null() {
            let live = LIVE.fetch_add(l.size(), Relaxed) + l.size();
            PEAK.fetch_max(live, Relaxed);
            TOTAL.fetch_add(l.size(), Relaxed);
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) };
        LIVE.fetch_sub(l.size(), Relaxed);
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Allocation {
    /// Bytes allocated while the closure ran.
    pub allocated_bytes: usize,
    /// Peak live bytes above the live bytes at the start.
    pub peak_extra_bytes: usize,
    /// Live bytes still held by the closure's result when it returned.
    pub retained_bytes: usize,
}

/// Allocation accounting of `f` (zeros unless the binary installs `Counting`). The result is kept alive until counted.
pub fn allocation<R>(f: impl FnOnce() -> R) -> (R, Allocation) {
    let start = LIVE.load(Relaxed);
    PEAK.store(start, Relaxed);
    let total = TOTAL.load(Relaxed);
    let r = f();
    let a = Allocation { allocated_bytes: TOTAL.load(Relaxed) - total, peak_extra_bytes: PEAK.load(Relaxed).saturating_sub(start),
                         retained_bytes: LIVE.load(Relaxed).saturating_sub(start) };
    (r, a)
}
