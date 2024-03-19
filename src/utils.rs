use std::sync::Mutex;

use anyhow::Result;
use lazy_static::lazy_static;
use rand::{distributions::Uniform, thread_rng, Rng};
use statrs::distribution::{Continuous, Normal};
use tokio::time::{sleep_until, Duration, Instant};

lazy_static! {
    static ref IDS: Mutex<Vec<usize>> = Mutex::new(Vec::new());
}

// Generate random id
pub fn generate_unique_id() -> usize {
    let mut ids = IDS.lock().unwrap();

    let mut rng = thread_rng();
    let mut val = rng.gen();

    while ids.contains(&val) {
        val = rng.gen();
    }

    ids.push(val);

    val
}

// Generate value in [500; 1000]
pub fn generate_work_time() -> u64 {
    let mut rng = thread_rng();
    let side = Uniform::new(500, 1000);

    rng.sample(side)
}

pub fn generate_chance_of_accident() -> Result<bool> {
    let mut rng = thread_rng();
    let side = Uniform::new(0., 1.);

    let x = rng.sample(side);
    let dist = Normal::new(0., 1.)?;

    Ok(dist.pdf(x) > 0.26)
}

pub async fn imitation_of_work() {
    let time = generate_work_time();
    sleep_until(Instant::now() + Duration::from_millis(time)).await;
}
