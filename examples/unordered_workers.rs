use env_logger::fmt::TimestampPrecision;
use env_logger::Env;
use futures::stream::FuturesUnordered;
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

    // Shortest option: create a collection of unordered futures and `join_all(...).await` on it.
    let tasks: FuturesUnordered<_> = (1..=5).map(|x| tokio::spawn(sleep_worker(x))).collect();
    // `FuturesUnordered` implements `IntoIterator` so it coerces back into a vector anyway.
    // Thus we can just construct a vector of futures like the line commented out below.
    //let tasks: Vec<_> = (1..=5).map(|x| tokio::spawn(sleep_worker(x))).collect();
    let results: Vec<Result<u64, _>> = futures::future::join_all(tasks).await;
    println!("{:?}", results);
    // ...or do `.collect().await`:
    //let _results: Vec<_> = tasks.collect().await;

    // Alternative #1: use `select_next_some` from Futures' `StreamExt`.
    // use futures::stream::StreamExt;
    // use futures::select;
    // let mut completed = 0;
    // loop {
    //     select! {
    //         _num = tasks.select_next_some() => {
    //             completed += 1;
    //         },
    //         complete => break,
    //     }
    // }

    // Alternative #2: directly populate workers and use `join_all`.
    // (NOTE: using a range with `.map` doesn't work for reasons I didn't pursue.)
    // let tasks = vec![
    //     tokio::spawn(sleep_worker(1)),
    //     tokio::spawn(sleep_worker(2)),
    //     tokio::spawn(sleep_worker(3)),
    //     tokio::spawn(sleep_worker(4)),
    //     tokio::spawn(sleep_worker(5)),
    // ];
    // futures::future::join_all(tasks).await;

    // Alternative #3: just use `for` loops. 🤷
    // let mut handles = Vec::new();
    // for i in 1..=5 {
    //     handles.push(tokio::spawn(sleep_worker(i)));
    // }

    // let mut output = Vec::new();
    // for handle in handles {
    //     output.push(handle.await.unwrap());
    // }
    // println!("{:?}", output);
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
