use async_stream::stream;
use env_logger::fmt::TimestampPrecision;
use env_logger::Env;
use futures::stream::{SelectAll, StreamExt};
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
    let items = [("A", 2), ("B", 3), ("C", 4)];
    let stream_producers = stream! {
        for (name, i) in items {
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

            yield ((name, i), UnboundedReceiverStream::new(rx));
        }
    };
    futures::pin_mut!(stream_producers);

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

    // NOTE: When using `tokio::select!` we have to match on `Some(expected_value)`.
    // However, also matching on `None` should NOT be done because that leads to early exits
    // e.g. we produced all streams and are now trying to get more but we get none
    // and we don't proceed to actually read data from the streams themselves
    // (the second arm of the `tokio::select!` construct).

    // Hence, when using `tokio::select!` we should only match on what we expect to come out
    // and ignore everything else?

    // TODO: Make stream producing fallible so we learn how to handle errors.
    // TODO: Make reading from a stream fallible so we learn how to handle errors there as well.

    // loop {
    //     tokio::select! {
    //         Some(((name, i), stream)) = stream_producers.next() => {
    //             trace!("Produced stream {}_{}", name, i);
    //             streams.push(stream);
    //         },
    //         Some(value) = streams.next() => {
    //             trace!("Received {}", value);
    //         },
    //         else => break,
    //     }
    // }
}

#[tokio::main]
async fn main() {
    init_logging();

    let time = Instant::now();
    exercise_add_workers_during_runtime().await;
    trace!("Program took {:?}", time.elapsed());
}
