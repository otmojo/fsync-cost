use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::os::fd::AsRawFd;  // <-- 添加这行，Rust 1.75+ 需要
use std::time::Instant;

const FILE_PATH: &str = "/home/leboun/test.db";
const WRITE_SIZE: usize = 4096;
const TOTAL_WRITES: usize = 100_000;
const WARMUP_WRITES: usize = 10_000;
const BATCH_SIZE: usize = 1000;  // 改成 10, 100, 1000 分别跑

fn main() -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(FILE_PATH)?;

    let data = vec![b'a'; WRITE_SIZE];

    // 预热
    println!("Warming up...");
    for i in 0..WARMUP_WRITES {
        file.write_all(&data)?;
        if i % 1000 == 0 {
            print!(".");
            std::io::stdout().flush()?;
        }
    }
    // 预热结束后刷一次
    unsafe { libc::fdatasync(file.as_raw_fd()) };
    println!("\nWarmup done.");

    // 正式测试
    println!("Starting benchmark with BATCH_SIZE={}...", BATCH_SIZE);
    let start = Instant::now();

    for i in 0..TOTAL_WRITES {
        file.write_all(&data)?;
        
        // 每 BATCH_SIZE 次写后刷一次
        if (i + 1) % BATCH_SIZE == 0 {
            unsafe { libc::fdatasync(file.as_raw_fd()) };
        }

        if i % 1000 == 0 {
            print!(".");
            std::io::stdout().flush()?;
        }
    }
    
    // 最后刷一次，确保剩余数据落盘
    unsafe { libc::fdatasync(file.as_raw_fd()) };

    let duration = start.elapsed();
    let total_secs = duration.as_secs_f64();
    let ops_per_sec = TOTAL_WRITES as f64 / total_secs;
    let avg_latency_ms = (total_secs / TOTAL_WRITES as f64) * 1000.0;

    println!("\n--- Results ---");
    println!("Total time: {:.3} sec", total_secs);
    println!("Ops/sec: {:.0}", ops_per_sec);
    println!("Avg latency: {:.3} ms", avg_latency_ms);

    Ok(())
}