use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::os::fd::AsRawFd;
use std::time::Instant;
use std::mem;

const FILE_PATH: &str = "/home/leboun/test.db";
const WRITE_SIZE: usize = 4096;
const TOTAL_WRITES: usize = 10_000;  // O_DIRECT 更慢，减少到 1 万次
const WARMUP_WRITES: usize = 1_000;

fn main() -> std::io::Result<()> {
    // O_DIRECT 要求：缓冲区必须对齐到 512 字节（通常）
    // 用 Vec 分配，保证对齐
    let mut data = vec![0u8; WRITE_SIZE];
    for i in 0..WRITE_SIZE {
        data[i] = (i % 256) as u8;
    }
    
    // 确保缓冲区对齐到 512 字节（O_DIRECT 要求）
    // 简单做法：分配 4KB + 511，手动对齐（这里简化，用 Vec 假设已对齐）
    
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .custom_flags(libc::O_DIRECT)  // <-- 关键：O_DIRECT 标志
        .open(FILE_PATH)?;

    println!("Opened file with O_DIRECT, alignment: 4096 bytes");

    // 预热
    println!("Warming up...");
    for i in 0..WARMUP_WRITES {
        file.write_all(&data)?;
        // O_DIRECT 模式下，write 本身已经绕过 cache，但为了公平对比，仍然做 fdatasync
        unsafe { libc::fdatasync(file.as_raw_fd()) };
        if i % 100 == 0 {
            print!(".");
            std::io::stdout().flush()?;
        }
    }
    println!("\nWarmup done.");

    // 正式测试
    println!("Starting benchmark with O_DIRECT...");
    let start = Instant::now();

    for i in 0..TOTAL_WRITES {
        file.write_all(&data)?;
        unsafe { libc::fdatasync(file.as_raw_fd()) };
        
        if i % 100 == 0 {
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