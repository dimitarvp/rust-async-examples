use env_logger::fmt::TimestampPrecision;
use env_logger::Env;
use log::trace;
use tokio::sync::watch;
use tokio::time::Instant;
use tokio_stream::{wrappers::WatchStream, StreamExt};

fn init_logging() {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace"))
        .format_timestamp(Some(TimestampPrecision::Nanos))
        .init();
}

async fn exercise_watch_stream() {
    let (tx, rx) = watch::channel("first message");
    let mut rx = WatchStream::new(rx);
    tx.send("second message").unwrap();
    tx.send("third message").unwrap();
    trace!("Sent three messages to WatchStream");
    trace!(
        "Expected to only get 'third message' from the WatchStream: {}",
        rx.next().await == Some("third message")
    );
}

#[tokio::main]
async fn main() {
    init_logging();

    let time = Instant::now();
    exercise_watch_stream().await;
    trace!("Program took {:?}", time.elapsed());
}
