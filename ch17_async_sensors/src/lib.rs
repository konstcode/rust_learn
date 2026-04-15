// ## Chapter 17 Task: Async Sensor Data Pipeline
//
// Build a self-contained async program simulating a sensor network.
//
// ---
//
// ### Scenario
//
// Multiple temperature sensors run as independent async tasks, emit readings through a shared channel, and a central collector processes them as a stream.
//
// ---
//
// ### Requirements
//
// **1. Sensor producers**
//
// Implement `async fn run_sensor(id: u32, tx: Sender<SensorReading>)`. Each sensor emits 5–10 readings with short sleeps between them. **One sensor (id=2) is slow** — it stalls longer than your timeout before sending its last reading.
//
// `SensorReading` must carry: sensor id, temperature (`f64`), sequence number.
//
// **2. Spawning**
//
// Spawn at least **4 sensors** with `tokio::spawn`. Use `join!` (not sequential `.await`) to await at least two `JoinHandle`s together.
//
// **3. Timeout handling**
//
// For the slow sensor, use `tokio::time::timeout` or `select!` to race it against a deadline. When it fires, log with `eprintln!` and proceed. The slow sensor must NOT hang the pipeline.
//
// **4. Stream processing**
//
// Drop all `tx` clones when producers finish so the receiver terminates naturally. Then:
// - Wrap `Receiver` in `ReceiverStream`
// - Apply `filter` — keep only readings where `-50.0 <= temp <= 150.0`
// - Apply `map` — convert each reading to a formatted `String`
// - `.collect::<Vec<String>>().await`
//
// **5. Final report**
//
// Print: total readings received, how many passed the filter, and the formatted list.
//
// ---
//
// ### Cargo.toml
//
// ```toml
// [dependencies]
// tokio = { version = "1", features = ["full"] }
// tokio-stream = { version = "0.1", features = ["full"] }
// ```
//
// ---
//
// ### Key types
//
// ```
// tokio::sync::mpsc::{channel, Sender, Receiver}
// tokio_stream::wrappers::ReceiverStream
// tokio_stream::StreamExt          // .filter(), .map(), .collect()
// tokio::time::{sleep, timeout, Duration}
// tokio::join!
// tokio::select!
// ```
//
// ---
//
// ### Hints
//
// - **Hint 1 — stream termination:** every `tx` clone must be dropped for `ReceiverStream` to return `None`. `tokio::spawn` moves ownership into the task — when the task finishes, its `tx` clone is dropped. Don't accidentally hold an extra clone in `main`.
// - **Hint 2 — collect type:** `let results: Vec<String> = stream.collect().await` — the type annotation is enough for inference.
//
// ---
//
// ### Stretch goal
//
// After collecting, spawn a second task that concurrently computes min/max temperature from raw readings. Use `join!` to drive both the stream collection and min/max computation concurrently.
//
// ---
//
// ### Success criteria
//
// - At least 4 sensors via `tokio::spawn`
// - Slow sensor triggers a visible timeout message
// - `join!` used for multiple handles (not sequential `.await`)
// - `ReceiverStream` used to consume the channel as a stream
// - Stream chain has both `filter` and `map`
// - Final output shows accepted reading count and formatted summaries

use std::fmt::Display;

use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::{Duration, sleep, timeout};
use tokio_stream::StreamExt; // .filter(), .map(), .collect()
use tokio_stream::wrappers::ReceiverStream;

#[derive(Debug)]
struct SensorReading {
    id: u32,
    temperature: f64,
    sequence_num: i32,
}

impl Display for SensorReading {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Sensor id({}) #{}: {}",
            self.id, self.sequence_num, self.temperature
        )
    }
}

async fn run_sensor(id: u32, tx: Sender<SensorReading>) {
    let max: i32 = rand::random_range(5..=10);
    for sequence_num in 0..=max {
        let temp: f64 = rand::random_range(-70.0..=180.0);
        let readings = SensorReading {
            id,
            temperature: temp,
            sequence_num,
        };

        // special case for 2nd sensor, last readings
        if id == 2 && sequence_num == max {
            sleep(Duration::from_millis(500)).await;
        } else {
            sleep(Duration::from_millis(100)).await;
        }

        let _ = tx.send(readings).await;
    }
}

async fn run_simulation() {
    let (tx, rx) = mpsc::channel(32);
    let mut handlers = vec![];
    for i in 1..=4 {
        let tx2 = tx.clone();
        handlers.push(tokio::spawn(async move {
            run_sensor(i, tx2).await;
        }));
    }

    if timeout(Duration::from_millis(300), handlers.remove(1))
        .await
        .is_err()
    {
        eprintln!("did not receive from 2nd sensor readings within 300 ms");
    }

    let sensors_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    let sensor_output = sensors_stream
        .filter(|s| -50.0 <= s.temperature && s.temperature <= 150.0)
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
        .await;

    let (first, second) = tokio::join!(handlers.remove(0), handlers.remove(2));
    drop(tx);
}
