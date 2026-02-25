use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::os::fd::AsRawFd;
use std::time::Instant;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    const FILE_PATH: &str = "/home/leboun/test.db";  // 确认路径是你的 home
    const WRITE_SIZE: usize = 4096;                  // 4KB
    const TOTAL_WRITES: usize = 100_000;
    const WARMUP_WRITES: usize = 10_000;

    // 1. 打开文件，不使用 O_DIRECT（先测正常 page cache 行为）
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(FILE_PATH)?;

    // 准备 4KB 数据（可重复模式，避免全零被优化）
    let data = vec![b'a'; WRITE_SIZE];

    // 2. 预热
    println!("Warming up...");
    for i in 0..WARMUP_WRITES {
        file.write_all(&data)?;
        unsafe { libc::fdatasync(file.as_raw_fd()) };
        if i % 1000 == 0 {
            print!(".");
            std::io::stdout().flush()?;
        }
    }
    println!("\nWarmup done.");

    // 3. 正式测试
    println!("Starting benchmark...");
    let start = Instant::now();

    for i in 0..TOTAL_WRITES {
        file.write_all(&data)?;
        
        let start_fsync = Instant::now();
        unsafe { libc::fdatasync(file.as_raw_fd()) };
        let fsync_duration = start_fsync.elapsed();
        
        if fsync_duration > Duration::from_millis(100) {
            println!("\n⚠️ Slow fsync at {}: {:.2}ms", i, fsync_duration.as_millis());
        }
        
        if i % 1000 == 0 {
            print!(".");
            std::io::stdout().flush()?;
        }
    }

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