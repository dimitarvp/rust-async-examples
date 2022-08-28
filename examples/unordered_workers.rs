use env_logger::fmt::TimestampPrecision;
use env_logger::Env;
use futures::select;
use futures::stream::{FuturesUnordered, StreamExt};
use log::trace;
use tokio::time::{sleep, Duration, Instant};

async fn sleep_worker(i: u64) -> u64 {
    let time = Instant::now();
    sleep(Duration::from_millis(100 / i)).await;
    trace!("Worker {} elapsed: {:?}", i, time.elapsed());
    i
}

async fn exercise_out_of_order_execution() {
    // This program demonstrates executing async workers out of their original order
    // because they take different amounts of real time to finish.

    let mut tasks: FuturesUnordered<_> = (1..=5)
        .map(|x| tokio::spawn(async move { sleep_worker(x).await }))
        .collect();
    let mut completed = 0;
    loop {
        select! {
            _num = tasks.select_next_some() => {
                completed += 1;
            },
            complete => break,
        }
    }

    trace!("Completed {} workers", completed);

    // Alternative implementation only using `join_all`.
    // let tasks = vec![
    //     sleep_worker(1),
    //     sleep_worker(2),
    //     sleep_worker(3),
    //     sleep_worker(4),
    //     sleep_worker(5),
    // ];
    // futures::future::join_all(tasks).await;
}

fn init_logging() {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace"))
        .format_timestamp(Some(TimestampPrecision::Nanos))
        .init();
}

#[tokio::main]
async fn main() {
    init_logging();

    let time = Instant::now();
    exercise_out_of_order_execution().await;
    trace!("Program took {:?}", time.elapsed());
}
