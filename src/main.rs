use rand::prelude::*;
use tokio::time::{sleep, Duration, Instant};

type MakeRequestResult = Result<String, String>;

async fn make_request() -> MakeRequestResult {
    let t: u64 = rand::thread_rng().gen_range(3..9);
    sleep(Duration::from_secs(t)).await;
    if t % 2 == 0 {
        Ok("ok".to_string())
    } else {
        Err("err".to_string())
    }
}

#[derive(Debug)]
struct MakeRequestMeasures {
    result: Result<String, String>,
    delta: u64,
}

async fn make_request_measured() -> MakeRequestMeasures {
    let before = Instant::now();
    let result = make_request().await;
    let after = Instant::now();
    let delta = (after - before).as_secs();
    MakeRequestMeasures { result, delta }
}

struct Monkey {
    sequence_number: u32,
}
impl Monkey {
    fn new(sequence_number: u32) -> Self {
        Monkey { sequence_number }
    }
}

async fn make_monkey_work(monkey: Monkey) {
    let sequence_number = monkey.sequence_number;
    loop {
        let report = make_request_measured().await;
        println!("Monkey sequence number {sequence_number} report: {report:?}",)
    }
}

#[tokio::main]
async fn main() {
    for i in 0..8 {
        let monkey = Monkey::new(i);
        let _ = tokio::spawn(async move { make_monkey_work(monkey).await });
    }

    loop {
        sleep(Duration::from_secs(60)).await;
    }
}
