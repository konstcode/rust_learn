use crate::logs::{LogEntry, LogLevel};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};
use std::thread::{self, JoinHandle};

pub struct Stats {
    pub produced: AtomicU64,
    pub dropped: AtomicU64,
    pub written: AtomicU64,
}

pub struct LogManager {
    stats: Arc<Stats>,
    logs: Arc<Mutex<Vec<LogEntry>>>,
}

impl LogManager {
    /// Intitinlize log manager with specified number of threads.
    ///
    /// # Returns
    /// LogManager
    ///
    pub fn init() -> LogManager {
        LogManager {
            stats: Arc::new(Stats {
                produced: AtomicU64::new(0),
                dropped: AtomicU64::new(0),
                written: AtomicU64::new(0),
            }),
            logs: Arc::new(Mutex::new(vec![])),
        }
    }

    /// # Arguments
    ///
    /// * `producers` - count of threads to run producers
    ///
    /// To join
    /// threads.into_iter().for_each(|t| t.join().unwrap());
    pub fn execute(&self, producers: i32) {
        let (prod_sx, filter_rx) = mpsc::channel();
        let (filter_sx, transf_rx) = mpsc::channel();
        let (transf_sx, batcher_rx) = mpsc::channel();
        let (batcher_sx, aggreg_rx) = mpsc::channel();
        let mut threads_handlers = vec![];

        for p in 0..producers {
            threads_handlers.push(self.run_producer(p + 1, prod_sx.clone()));
        }
        drop(prod_sx);

        threads_handlers.push(self.run_filter(filter_sx, filter_rx));
        threads_handlers.push(self.run_transformer(transf_sx, transf_rx));
        threads_handlers.push(self.run_batcher(batcher_sx, batcher_rx));
        threads_handlers.push(self.run_aggregator(aggreg_rx));
        threads_handlers.into_iter().for_each(|h| {
            h.join().unwrap();
        });
    }

    pub fn get_stats(&self) -> Arc<Stats> {
        Arc::clone(&self.stats)
    }

    pub fn get_logs(&self) -> Arc<Mutex<Vec<LogEntry>>> {
        Arc::clone(&self.logs)
    }

    ///  - Each generates 20 log entries
    ///  - Use a shared AtomicU64 for globally unique IDs across all producers
    ///  - Distribute entries across all LogLevel variants
    ///  - Increment Stats::produced for each entry sent
    fn run_producer(&self, thread_id: i32, sender: Sender<LogEntry>) -> JoinHandle<()> {
        let levels = [
            LogLevel::Trace,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
        ];

        let stats = self.get_stats();
        thread::spawn(move || {
            for log in 0..20 {
                let id = stats.produced.fetch_add(1, Ordering::Relaxed);
                let message = format!("producer {} entry {}", thread_id, &id);
                let _ = sender.send(LogEntry::new(id, levels[log % levels.len()], message));
            }
        })
    }

    ///  - Receives from many producers via a single mpsc channel (clone senders)
    ///  - Discards Trace and Debug entries → increment Stats::dropped
    ///  - Forwards the rest to the transform stage
    fn run_filter(&self, sender: Sender<LogEntry>, receiver: Receiver<LogEntry>) -> JoinHandle<()> {
        let stats = self.get_stats();
        thread::spawn(move || {
            for log in receiver {
                if let LogLevel::Warn | LogLevel::Error | LogLevel::Info = log.level() {
                    let _ = sender.send(log);
                } else {
                    stats.dropped.fetch_add(1, Ordering::Relaxed);
                }
            }
        })
    }

    ///  - Uppercases message and appends " [processed]" to it
    ///  - Forwards each entry to the aggregator
    fn run_transformer(
        &self,
        sender: Sender<LogEntry>,
        receiver: Receiver<LogEntry>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            for mut log in receiver {
                log.set_message(format!("{} [processed]", log.message().to_uppercase()));
                let _ = sender.send(log);
            }
        })
    }

    ///  - Appends entries to a Vec<LogEntry> inside Arc<Mutex<Vec<LogEntry>>>
    ///  - Increments Stats::written for each entry
    fn run_aggregator(&self, receiver: Receiver<Vec<LogEntry>>) -> JoinHandle<()> {
        let stats = self.get_stats();
        let logs = Arc::clone(&self.logs);
        thread::spawn(move || {
            for logs_batch in receiver {
                let length: u64 = logs_batch.len().try_into().unwrap();
                logs.lock().unwrap().extend(logs_batch);
                stats.written.fetch_add(length, Ordering::Relaxed);
            }
        })
    }

    ///  Add a batch writer stage between transform and aggregator that collects entries into batches of 5 before forwarding.
    fn run_batcher(
        &self,
        sender: Sender<Vec<LogEntry>>,
        receiver: Receiver<LogEntry>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            let mut batch = vec![];
            for log in receiver {
                batch.push(log);
                if batch.len() >= 5 {
                    sender.send(batch).unwrap();
                    batch = vec![];
                }
            }

            if !batch.is_empty() {
                sender.send(batch).unwrap();
            }
        })
    }
}
