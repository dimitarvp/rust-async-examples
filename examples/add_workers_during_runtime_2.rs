use env_logger::fmt::TimestampPrecision;
use env_logger::Env;
use futures::stream::{self, SelectAll, StreamExt};
use log::{error, trace};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration, Instant};
use tokio_stream::wrappers::UnboundedReceiverStream;

fn init_logging() {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace"))
        .format_timestamp(Some(TimestampPrecision::Nanos))
        .init();
}

async fn exercise_add_workers_during_runtime() {
    let mut stream_producers = stream::iter([("A", 2), ("B", 3), ("C", 4)].map(|(name, i)| {
        let (tx, rx) = mpsc::unbounded_channel::<String>();

        tokio::spawn(async move {
            sleep(Duration::from_millis(480 / i)).await;

            for _ in 1..=i {
                sleep(Duration::from_millis(120 / i)).await;
                let to_send = format!("{}_{}", name, i);
                match tx.send(to_send.clone()) {
                    Ok(_) => trace!("Sent {}", to_send.clone()),
                    Err(e) => error!("{}", format!("{:?}", e)),
                }
            }
        });

        ((name, i), UnboundedReceiverStream::new(rx))
    }))
    .fuse();

    let mut streams = SelectAll::new();

    loop {
        futures::select! {
            ((name, i), stream) = stream_producers.select_next_some() => {
                trace!("Produced stream {}_{}", name, i);
                streams.push(stream);
            },
            value = streams.select_next_some() => {
                trace!("Received {}", value);
            },
            complete => break,
        }
    }
}

#[tokio::main]
async fn main() {
    init_logging();

    let time = Instant::now();
    exercise_add_workers_during_runtime().await;
    trace!("Program took {:?}", time.elapsed());
}
