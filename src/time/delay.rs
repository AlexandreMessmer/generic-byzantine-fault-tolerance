use std::time::Duration;

use rand::{thread_rng, Rng};

pub async fn transmition_delay() {
    let normal = rand_distr::Normal::new(1.0, 0.1).unwrap();
    tokio::time::sleep(Duration::from_nanos(
        (thread_rng().sample(normal) * 1000.0) as u64,
    ))
    .await;
}

pub async fn simulate_busy() {
    let mut range = thread_rng();
    let normal = rand_distr::Normal::new(1.0, 1.0).unwrap();
    tokio::time::sleep(Duration::from_millis(
        (range.sample(normal) * 1000.0) as u64,
    ))
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn sleep_as_expected() {
        for _ in 0..1000 {
            transmition_delay().await;
        }
    }

    #[tokio::test]
    async fn busy_as_expected() {
        simulate_busy().await;
        let mut range = thread_rng();
        let normal = rand_distr::Normal::new(1.0, 0.1).unwrap();
        println!(
            "{:?}",
            Duration::from_millis((range.sample(normal) * 1000.0) as u64)
        );
    }
}
