use super::scenarios::Scenarios;

const MAX_CONSENSUS: u64 = 2000;

async fn slowdown_factor(transmission_delay_ms: u64) {
    let mut scenarios = Scenarios::new(&format!("reports/scenarios/slowdown_factor_{}ms.txt", transmission_delay_ms));
    let low = 1;
    let step = 2;
    let high = MAX_CONSENSUS / transmission_delay_ms;
    scenarios.slowdown_factor(low, high, step, transmission_delay_ms).await;
}

#[tokio::test]
async fn slowdown_factor_100ms() {
    slowdown_factor(100).await;
}

#[tokio::test]
async fn slowdown_factor_300ms() {
    slowdown_factor(300).await;
}

#[tokio::test]
async fn slowdown_factor_50ms() {
    slowdown_factor(50).await;
}

#[tokio::test]
async fn slowdown_factor_200ms() {
    slowdown_factor(200).await;
}
