use tokio::time::{sleep, Duration};

struct Monkey {
    sequence_number: u32,
}
impl Monkey {
    fn new(sequence_number: u32) -> Self {
        Monkey { sequence_number }
    }
}

async fn make_monkey_work(monkey: Monkey) {
    loop {
        println!("Monkey's sequence number is {}", monkey.sequence_number);
        sleep(Duration::from_secs(8)).await;
    }
}

#[tokio::main]
async fn main() {
    for i in 0..8 {
        let monkey = Monkey::new(i);
        let _ = tokio::spawn(async move { make_monkey_work(monkey).await });
        sleep(Duration::from_secs(1)).await;
    }

    loop {
        sleep(Duration::from_secs(60)).await;
    }
}
