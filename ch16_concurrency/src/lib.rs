/*
 * file: lib.rs
 * author: Kostiantyn Konakhevych
 * date: 2026-03-29
 */

//!  Data Types
//!
//!  enum LogLevel { Trace, Debug, Info, Warn, Error }
//!
//!  struct LogEntry {
//!      id: u64,
//!      level: LogLevel,
//!      message: String,
//!      timestamp_ms: u64,
//!  }
//!
//!  struct Stats {
//!      produced: AtomicU64,
//!      dropped: AtomicU64,
//!      written: AtomicU64,
//!  }
//!
//!  Pipeline
//!
//!  [Producer 1] ─┐
//!  [Producer 2] ──→ [Filter] → [Transform] → [Aggregator]
//!  [Producer 3] ─┘
//!
//!  Requirements
//!
//!  1. Producer threads (×3)
//!  - Each generates 20 log entries → 60 total
//!  - Use a shared AtomicU64 for globally unique IDs across all producers
//!  - Distribute entries across all LogLevel variants
//!  - Increment Stats::produced for each entry sent
//!
//!  2. Filter stage (×1 thread)
//!  - Receives from all 3 producers via a single mpsc channel (clone senders)
//!  - Discards Trace and Debug entries → increment Stats::dropped
//!  - Forwards the rest to the transform stage
//!
//!  3. Transform stage (×1 thread)
//!  - Uppercases message and appends " [processed]" to it
//!  - Forwards each entry to the aggregator
//!
//!  4. Aggregator thread (×1 thread)
//!  - Appends entries to a Vec<LogEntry> inside Arc<Mutex<Vec<LogEntry>>>
//!  - Increments Stats::written for each entry
//!
//!  5. Shutdown
//!  - All threads joined in order: producers → filter → transform → aggregator
//!  - No detached threads
//!
//!  6. Final report
//!  - Print produced, dropped, written counts
//!  - Assert produced == dropped + written
//!  - Print every final entry (id, level, message)
//!
//!  Acceptance Criteria
//!
//!  - No warnings
//!  - produced == dropped + written holds every run
//!  - Stats uses AtomicU64 directly — no Mutex<u64>
//!  - All threads explicitly joined
//!  - Messages are uppercased and end with " [processed]" in output
//!
//!  Stretch goal
//!
//!  Add a batch writer stage between transform and aggregator that collects entries into batches of 5 before forwarding.

pub mod log_manager;
pub mod logs;
