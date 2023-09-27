use rand::prelude::*;
use tokio::time::{sleep, Duration, Instant};
use tokio::sync::{mpsc, oneshot};
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicU8;

type MakeRequestResult = bool;

#[derive(Debug)]
struct MakeRequestMeasures {
    result: MakeRequestResult,
    delta: u64,
}

struct Monkey {
    sequence_number: u32,
}
impl Monkey {
    fn new(sequence_number: u32) -> Self {
        Monkey { sequence_number }
    }
}

async fn generate_resouces(pool: Arc<Mutex<u8>>) {
    loop {
        if let Ok(mut  value) = pool.lock() {
            *value = value.saturating_add(1);
        }
        sleep(Duration::from_millis(100)).await;
    }
}

async fn process_request(mut rx: mpsc::Receiver<oneshot::Sender<bool>>) {
    let pool = Arc::new(Mutex::new(0u8));
    let generator_pool = pool.clone();
    let _ = tokio::spawn(async move { generate_resouces(generator_pool).await });
    while let Some(tx) = rx.recv().await {
        let mut counter = 0;
        if let Ok(mut  value) = pool.lock() {
            counter = value.saturating_sub(1);
            *value = counter
        }
        println!("counter = {counter}");
        tx.send(counter != 0).unwrap();
    }
}

async fn make_request(mpsc_tx: mpsc::Sender<oneshot::Sender<bool>>) -> MakeRequestResult {
    let (oneshot_tx, oneshot_rx) = oneshot::channel::<bool>();
    mpsc_tx.send(oneshot_tx).await.unwrap();
    oneshot_rx.await.unwrap()
}

async fn make_request_measured(mpsc_tx: mpsc::Sender<oneshot::Sender<bool>>) -> MakeRequestMeasures {
    let before = Instant::now();
    let result = make_request(mpsc_tx).await;
    let after = Instant::now();
    let delta = (after - before).as_secs();
    MakeRequestMeasures { result, delta }
}

async fn make_monkey_work(monkey: Monkey, mpsc_tx: mpsc::Sender<oneshot::Sender<bool>>) {
    let sequence_number = monkey.sequence_number;
    loop {
        sleep(Duration::from_secs(1)).await;
        let report = make_request_measured(mpsc_tx.clone()).await;
        println!("Monkey sequence number {sequence_number} report: {report:?}",)
    }
}

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel::<oneshot::Sender::<bool>>(32);

    let _ = tokio::spawn(async move { process_request(rx).await });

    for i in 0..12 {
        let monkey = Monkey::new(i);
        let mpsc_tx = tx.clone();
        let _ = tokio::spawn(async move { make_monkey_work(monkey, mpsc_tx).await });
        sleep(Duration::from_millis(100)).await;
    }

    loop {
        sleep(Duration::from_secs(60)).await;
    }
}
