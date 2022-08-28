use env_logger::fmt::TimestampPrecision;
use env_logger::Env;
use futures::select;
use futures::stream::{FuturesUnordered, StreamExt};
use log::{error, trace};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::time::timeout;
use tokio::time::{sleep, Duration, Instant};
use tokio_stream as stream;

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

async fn channel_sending_worker(i: u64, tx: UnboundedSender<String>) {
    let mut stream = stream::iter(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    while let Some(value) = stream.next().await {
        let to_send = format!("{}_{}", i, value);
        match tx.send(to_send.clone()) {
            Ok(_) => trace!("Sent {}", to_send.clone()),
            Err(e) => error!("{}", format!("{:?}", e)),
        }
        sleep(Duration::from_millis(51)).await
    }

    trace!("Worker {} successfully sent all data", i);
}

async fn channel_receiving_worker(rx: &mut UnboundedReceiver<String>) {
    let expected_amount = 50;
    let mut consumed_amount = 0;

    while let Some(value) = rx.recv().await {
        consumed_amount += 1;
        trace!("Received {}", value);
        if consumed_amount == expected_amount {
            rx.close();
            trace!("Receiver has sucessfully fetched all data");
        }
    }

    if consumed_amount != expected_amount {
        error!(
            "Receiver fetched {} pieces of data, {} were expected",
            consumed_amount, expected_amount
        );
    }
}

async fn exercise_parallel_channels_and_deadline() {
    // This program demonstrates several parallel workers sending messages to channel
    // and a central worker accepting and printing them. Everything shuts down
    // after a deadline passes.

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let tasks = FuturesUnordered::new();
    for i in 1..=5 {
        let tx = tx.clone();
        tasks.push(tokio::spawn(
            async move { channel_sending_worker(i, tx).await },
        ));
    }
    tasks.push(tokio::spawn(async move {
        channel_receiving_worker(&mut rx).await
    }));

    let futures = futures::future::join_all(tasks.into_iter());
    let future = timeout(Duration::from_millis(100), futures);

    match future.await {
        Ok(_) => {
            trace!("All tasks done before deadline has expired");
        }
        Err(_) => {
            trace!("Deadline has expired");
        }
    }
}

fn init() {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace"))
        .format_timestamp(Some(TimestampPrecision::Nanos))
        .init();
}

#[tokio::main]
async fn main() {
    init();

    let time = Instant::now();
    //exercise_out_of_order_execution().await;
    exercise_parallel_channels_and_deadline().await;
    trace!("Program took {:?}", time.elapsed());
}
