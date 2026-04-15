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

use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::{Duration, sleep, timeout};
use tokio_stream::StreamExt; // .filter(), .map(), .collect()
use tokio_stream::wrappers::ReceiverStream;

struct SensorReading {
    id: u32,
    temperature: f64,
    sequence_num: i32,
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

        // special case for 2nd sensor
        if id == 2 && sequence_num == max {
            sleep(Duration::from_millis(500)).await;
        } else {
            sleep(Duration::from_millis(100)).await;
        }

        let _ = tx.send(readings).await;
    }
}
