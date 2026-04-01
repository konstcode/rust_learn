/*
 * file: main.rs
 * author: Kostiantyn Konakhevych
 * date: 2026-03-29
 */

//! Check lib.rs for main logic.
//!

use ch16_concurrency::log_manager::LogManager;
use std::sync::atomic::Ordering;

fn main() {
    let log_manager = LogManager::init();
    log_manager.execute(3);
    let stats = log_manager.get_stats();

    let (produced, dropped, written) = (
        stats.produced.load(Ordering::Relaxed),
        stats.dropped.load(Ordering::Relaxed),
        stats.written.load(Ordering::Relaxed),
    );
    println!("Stats:");
    println!("Produced: {}", produced);
    println!("Dropped: {}", dropped);
    println!("Written: {}", written);

    assert!(produced == dropped + written);
    let logs = log_manager.get_logs();
    for log in logs.lock().unwrap().iter() {
        println!("{}", log);
    }
}
