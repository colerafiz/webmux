use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use bytes::{Bytes, BytesMut};
use std::time::Duration;
use tokio::runtime::Runtime;

// Benchmark terminal output processing
fn benchmark_terminal_output_processing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("terminal_output");
    group.measurement_time(Duration::from_secs(10));
    
    // Test different output sizes
    for size in [1024, 4096, 16384, 65536, 262144].iter() {
        let data = vec![b'x'; *size];
        
        group.throughput(Throughput::Bytes(*size as u64));
        
        // Benchmark JSON encoding (old approach)
        group.bench_with_input(BenchmarkId::new("json_encoding", size), size, |b, _| {
            b.iter(|| {
                let output = serde_json::json!({
                    "type": "output",
                    "data": String::from_utf8_lossy(&data)
                });
                let _ = serde_json::to_string(&output).unwrap();
            });
        });
        
        // Benchmark binary encoding (new approach)
        group.bench_with_input(BenchmarkId::new("binary_encoding", size), size, |b, _| {
            b.iter(|| {
                let mut buffer = BytesMut::with_capacity(data.len() + 5);
                buffer.extend_from_slice(&[0x01]); // Message type
                buffer.extend_from_slice(&(data.len() as u32).to_le_bytes());
                buffer.extend_from_slice(&data);
                let _ = buffer.freeze();
            });
        });
    }
    
    group.finish();
}

// Benchmark message batching
fn benchmark_message_batching(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("message_batching");
    
    // Test different batch sizes
    for batch_size in [1, 10, 50, 100, 500].iter() {
        let messages: Vec<String> = (0..*batch_size)
            .map(|i| format!("Message {}", i))
            .collect();
        
        group.throughput(Throughput::Elements(*batch_size as u64));
        
        // Benchmark individual sends (old approach)
        group.bench_with_input(BenchmarkId::new("individual_sends", batch_size), &messages, |b, msgs| {
            b.iter(|| {
                for msg in msgs {
                    let _ = serde_json::to_string(&serde_json::json!({
                        "type": "output",
                        "data": msg
                    })).unwrap();
                }
            });
        });
        
        // Benchmark batched sends (new approach)
        group.bench_with_input(BenchmarkId::new("batched_sends", batch_size), &messages, |b, msgs| {
            b.iter(|| {
                let mut combined = String::with_capacity(msgs.iter().map(|m| m.len()).sum());
                for msg in msgs {
                    combined.push_str(msg);
                }
                let _ = serde_json::to_string(&serde_json::json!({
                    "type": "output",
                    "data": combined
                })).unwrap();
            });
        });
    }
    
    group.finish();
}

// Benchmark buffer operations
fn benchmark_buffer_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("buffer_operations");
    
    // Test different operation counts
    for ops in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*ops as u64));
        
        // Benchmark Vec-based buffer (old approach)
        group.bench_with_input(BenchmarkId::new("vec_buffer", ops), ops, |b, &ops| {
            b.iter(|| {
                let mut buffer = Vec::with_capacity(1024 * 1024);
                for i in 0..ops {
                    let data = format!("Line {}\n", i);
                    buffer.extend_from_slice(data.as_bytes());
                }
                black_box(buffer);
            });
        });
        
        // Benchmark BytesMut buffer (new approach)
        group.bench_with_input(BenchmarkId::new("bytes_buffer", ops), ops, |b, &ops| {
            b.iter(|| {
                let mut buffer = BytesMut::with_capacity(1024 * 1024);
                for i in 0..ops {
                    let data = format!("Line {}\n", i);
                    buffer.extend_from_slice(data.as_bytes());
                }
                black_box(buffer.freeze());
            });
        });
    }
    
    group.finish();
}

// Benchmark UTF-8 validation
fn benchmark_utf8_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("utf8_validation");
    
    // Test different string sizes
    for size in [1024, 16384, 131072, 1048576].iter() {
        let data = vec![b'a'; *size];
        
        group.throughput(Throughput::Bytes(*size as u64));
        
        // Benchmark standard UTF-8 validation
        group.bench_with_input(BenchmarkId::new("std_utf8", size), &data, |b, data| {
            b.iter(|| {
                let _ = std::str::from_utf8(data).unwrap();
            });
        });
        
        // Benchmark SIMD UTF-8 validation
        group.bench_with_input(BenchmarkId::new("simd_utf8", size), &data, |b, data| {
            b.iter(|| {
                let _ = simdutf8::basic::from_utf8(data).unwrap();
            });
        });
    }
    
    group.finish();
}

// Benchmark session management approaches
fn benchmark_session_management(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("session_management");
    group.sample_size(10); // Reduce sample size for slower operations
    
    // Simulate terminal output capture
    let terminal_content = "x".repeat(65536); // 64KB of content
    
    // Benchmark direct PTY attachment (simulated)
    group.bench_function("pty_attachment", |b| {
        b.iter(|| {
            // Simulate PTY read overhead
            std::thread::sleep(Duration::from_micros(100));
            black_box(&terminal_content);
        });
    });
    
    // Benchmark capture-pane approach (simulated)
    group.bench_function("capture_pane", |b| {
        b.iter(|| {
            // Simulate tmux capture-pane overhead
            std::thread::sleep(Duration::from_micros(50));
            black_box(&terminal_content);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_terminal_output_processing,
    benchmark_message_batching,
    benchmark_buffer_operations,
    benchmark_utf8_validation,
    benchmark_session_management
);
criterion_main!(benches);