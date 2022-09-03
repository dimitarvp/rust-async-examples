use env_logger::fmt::TimestampPrecision;
use env_logger::Env;
use futures::stream::select_all;
use futures::stream::FuturesUnordered;
use futures::stream::StreamExt;
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
    // Make a shared reference to a list of futures and fill them up
    // while a main receiver loop is already trying to wait on them.

    let mut stream_producers: FuturesUnordered<_> = [("A", 2), ("B", 3), ("C", 4)]
        .map(|(name, i)| {
            // We don't seem to need `tokio::spawn` (namely true parallelism) here
            // because the different tasks yield at different times.
            tokio::spawn(async move {
                let (tx, rx) = mpsc::unbounded_channel::<String>();
                sleep(Duration::from_millis(480 / i)).await;

                async move {
                    for _ in 1..=i {
                        sleep(Duration::from_millis(120 / i)).await;
                        let to_send = format!("{}_{}", name, i);
                        match tx.send(to_send.clone()) {
                            Ok(_) => trace!("Sent {}", to_send.clone()),
                            Err(e) => error!("{}", format!("{:?}", e)),
                        }
                    }
                }
                .await;

                ((name, i), UnboundedReceiverStream::new(rx))
            })
        })
        .into_iter()
        .collect();

    let mut streams = select_all(FuturesUnordered::new());

    loop {
        futures::select! {
            stream_result = stream_producers.select_next_some() => {
                match stream_result {
                    Ok(((name, i), stream)) => {
                        trace!("Produced stream {}_{}", name, i);
                        streams.push(stream);
                    },
                    Err(e) => {
                        error!("Could not get stream: {:?}", e);
                    }
                }
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
