use env_logger::fmt::TimestampPrecision;
use env_logger::Env;
use futures::stream::StreamExt;
use log::{error, trace};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{sleep, Duration, Instant};
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamMap};

fn init_logging() {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace"))
        .format_timestamp(Some(TimestampPrecision::Nanos))
        .init();
}

async fn exercise_add_workers_during_runtime() {
    // Make a shared reference to a list of futures and fill them up
    // while a main receiver loop is already trying to wait on them.

    let tasks = Arc::new(Mutex::new(StreamMap::new()));
    for (name, i) in vec![("A", 2), ("B", 3), ("C", 4)] {
        let tasks_ref = tasks.clone();
        tokio::spawn(async move {
            let (tx, rx) = mpsc::unbounded_channel::<String>();

            // Spawn sender.
            tokio::spawn(async move {
                for _ in 1..=i {
                    sleep(Duration::from_millis(120 / i)).await;
                    let to_send = format!("{}_{}", name, i);
                    match tx.send(to_send.clone()) {
                        Ok(_) => trace!("Sent {}", to_send.clone()),
                        Err(e) => error!("{}", format!("{:?}", e)),
                    }
                }
            });

            // Put receiver as a stream in the collections of futures.
            tasks_ref
                .lock()
                .await
                .insert((name, i), UnboundedReceiverStream::new(rx));
        });
    }

    while tasks.lock().await.is_empty() {
        trace!("No tasks in the queue, sleeping");
        sleep(Duration::from_millis(10)).await;
    }

    trace!("Start reader loop");

    while let Some(((_name, _i), value)) = tasks.lock().await.next().await {
        trace!("Received {}", &value);
    }
}

#[tokio::main]
async fn main() {
    init_logging();

    let time = Instant::now();
    exercise_add_workers_during_runtime().await;
    trace!("Program took {:?}", time.elapsed());
}
