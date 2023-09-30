use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
use tokio::time::{sleep, Duration};

type MakeRequestResult = bool;

// #[derive(Debug)]
// struct MakeRequestMeasures {
//     result: MakeRequestResult,
//     delta: u64,
// }

async fn generate_resouces(pool: Arc<Mutex<u8>>) {
    loop {
        if let Ok(mut value) = pool.lock() {
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
        if let Ok(mut value) = pool.lock() {
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

// async fn make_request_measured(
//     mpsc_tx: mpsc::Sender<oneshot::Sender<bool>>,
// ) -> MakeRequestMeasures {
//     let before = Instant::now();
//     let result = make_request(mpsc_tx).await;
//     let after = Instant::now();
//     let delta = (after - before).as_secs();
//     MakeRequestMeasures { result, delta }
// }

async fn make_monkey_work(mpsc_tx: mpsc::Sender<oneshot::Sender<bool>>, failing: &AtomicBool) {
    loop {
        sleep(Duration::from_secs(1)).await;
        let result = make_request(mpsc_tx.clone()).await;
        if !result {
            failing.store(true, Ordering::Relaxed);
        }
    }
}

async fn determine_throughput() {
    let (tx, rx) = mpsc::channel::<oneshot::Sender<bool>>(32);
    let _ = tokio::spawn(async move { process_request(rx).await });
    let mut handles = vec![];

    let failing = Arc::new(AtomicBool::new(false));
    let mut stop = false;
    loop {
        let mpsc_tx = tx.clone();
        let failing_arc = Arc::clone(&failing);

        if !stop {
            handles.push(tokio::spawn(async move {
                make_monkey_work(mpsc_tx, &failing_arc).await
            }));
        }

        if failing.load(Ordering::Acquire) {
            stop = true;
            let handle = handles.pop().unwrap();
            handle.abort();
            failing.store(false, Ordering::Relaxed);
        }

        sleep(Duration::from_secs(1)).await;

        println!("handles_count = {}", handles.len());
    }
}

#[tokio::main]
async fn main() {
    determine_throughput().await;

    loop {
        sleep(Duration::from_secs(60)).await;
    }
}
