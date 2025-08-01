use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use bytes::Bytes;
use tokio::sync::Notify;
use crossbeam::queue::ArrayQueue;
use simdutf8;

const RING_BUFFER_SIZE: usize = 1 << 20; // 1MB
const MAX_CHUNK_SIZE: usize = 65536;

pub struct TerminalRingBuffer {
    // Main ring buffer
    buffer: Box<[u8; RING_BUFFER_SIZE]>,
    // Write position (only written by producer)
    write_pos: AtomicU64,
    // Sequence number for ordering
    sequence: AtomicU64,
    // Notification for new data
    notify: Arc<Notify>,
    // Queue for completed chunks
    chunks: Arc<ArrayQueue<TerminalChunk>>,
}

#[derive(Clone)]
pub struct TerminalChunk {
    pub sequence: u64,
    pub data: Bytes,
}

#[derive(Clone)]
pub struct TerminalReader {
    // Last read sequence
    last_sequence: u64,
    // Shared chunks queue
    chunks: Arc<ArrayQueue<TerminalChunk>>,
    // Notification handle
    notify: Arc<Notify>,
}

impl TerminalRingBuffer {
    pub fn new() -> Self {
        Self {
            buffer: Box::new([0u8; RING_BUFFER_SIZE]),
            write_pos: AtomicU64::new(0),
            sequence: AtomicU64::new(0),
            notify: Arc::new(Notify::new()),
            chunks: Arc::new(ArrayQueue::new(1024)),
        }
    }
    
    pub fn write(&mut self, data: &[u8]) -> Result<(), &'static str> {
        if data.len() > MAX_CHUNK_SIZE {
            return Err("Data too large for single write");
        }
        
        let sequence = self.sequence.fetch_add(1, Ordering::AcqRel);
        let write_pos = self.write_pos.load(Ordering::Acquire) as usize;
        
        // Copy data to ring buffer
        let end_pos = (write_pos + data.len()) % RING_BUFFER_SIZE;
        
        if end_pos > write_pos {
            // Simple case: contiguous write
            self.buffer[write_pos..end_pos].copy_from_slice(data);
        } else {
            // Wrap around case
            let first_part = RING_BUFFER_SIZE - write_pos;
            self.buffer[write_pos..].copy_from_slice(&data[..first_part]);
            self.buffer[..end_pos].copy_from_slice(&data[first_part..]);
        }
        
        // Update write position
        self.write_pos.store(end_pos as u64, Ordering::Release);
        
        // Create chunk and add to queue
        let chunk = TerminalChunk {
            sequence,
            data: Bytes::copy_from_slice(data),
        };
        
        // Try to add to queue, if full, drop oldest
        while self.chunks.push(chunk.clone()).is_err() {
            let _ = self.chunks.pop();
        }
        
        // Notify readers
        self.notify.notify_waiters();
        
        Ok(())
    }
    
    pub fn create_reader(&self) -> TerminalReader {
        TerminalReader {
            last_sequence: 0,
            chunks: self.chunks.clone(),
            notify: self.notify.clone(),
        }
    }
}

impl TerminalReader {
    pub async fn read_next(&mut self) -> Option<TerminalChunk> {
        loop {
            // Try to find next chunk in sequence
            let mut found = None;
            for _ in 0..self.chunks.len() {
                if let Some(chunk) = self.chunks.pop() {
                    if chunk.sequence == self.last_sequence + 1 {
                        self.last_sequence = chunk.sequence;
                        found = Some(chunk);
                        break;
                    } else if chunk.sequence > self.last_sequence {
                        // Re-queue for later
                        let _ = self.chunks.push(chunk);
                    }
                    // Drop chunks with sequence <= last_sequence (already read)
                }
            }
            
            if let Some(chunk) = found {
                return Some(chunk);
            }
            
            // Wait for new data
            self.notify.notified().await;
        }
    }
    
    pub fn try_read_next(&mut self) -> Option<TerminalChunk> {
        // Non-blocking version
        for _ in 0..self.chunks.len() {
            if let Some(chunk) = self.chunks.pop() {
                if chunk.sequence == self.last_sequence + 1 {
                    self.last_sequence = chunk.sequence;
                    return Some(chunk);
                } else if chunk.sequence > self.last_sequence {
                    // Re-queue for later
                    let _ = self.chunks.push(chunk);
                }
            }
        }
        None
    }
}

// Zero-copy UTF-8 streaming decoder
pub struct Utf8StreamDecoder {
    incomplete: Vec<u8>,
}

impl Utf8StreamDecoder {
    pub fn new() -> Self {
        Self {
            incomplete: Vec::with_capacity(4),
        }
    }
    
    pub fn decode_chunk(&mut self, input: &[u8]) -> (String, usize) {
        let mut result = String::with_capacity(input.len());
        let mut processed = 0;
        
        // Handle incomplete bytes from previous chunk
        if !self.incomplete.is_empty() {
            let combined_len = self.incomplete.len() + input.len().min(4);
            let mut combined = Vec::with_capacity(combined_len);
            combined.extend_from_slice(&self.incomplete);
            combined.extend_from_slice(&input[..input.len().min(4)]);
            
            match simdutf8::basic::from_utf8(&combined) {
                Ok(s) => {
                    result.push_str(s);
                    processed = combined.len() - self.incomplete.len();
                    self.incomplete.clear();
                }
                Err(_) => {
                    // Try standard UTF-8 validation to get error position
                    match std::str::from_utf8(&combined) {
                        Ok(s) => {
                            result.push_str(s);
                            processed = combined.len() - self.incomplete.len();
                            self.incomplete.clear();
                        }
                        Err(e) => {
                            let valid_up_to = e.valid_up_to();
                            if valid_up_to > 0 {
                                result.push_str(unsafe { 
                                    std::str::from_utf8_unchecked(&combined[..valid_up_to]) 
                                });
                                processed = valid_up_to.saturating_sub(self.incomplete.len());
                                self.incomplete.clear();
                            }
                        }
                    }
                }
            }
        }
        
        // Process main input with SIMD acceleration
        let remaining = &input[processed..];
        match simdutf8::basic::from_utf8(remaining) {
            Ok(s) => {
                result.push_str(s);
                processed = input.len();
            }
            Err(_) => {
                // Try standard UTF-8 validation to get error position
                match std::str::from_utf8(remaining) {
                    Ok(s) => {
                        result.push_str(s);
                        processed = input.len();
                    }
                    Err(e) => {
                        let valid_up_to = e.valid_up_to();
                        if valid_up_to > 0 {
                            result.push_str(unsafe { 
                                std::str::from_utf8_unchecked(&remaining[..valid_up_to]) 
                            });
                        }
                        
                        // Save incomplete bytes for next chunk
                        let incomplete_start = processed + valid_up_to;
                        if incomplete_start < input.len() {
                            self.incomplete.clear();
                            self.incomplete.extend_from_slice(&input[incomplete_start..]);
                        }
                        processed = input.len();
                    }
                }
            }
        }
        
        (result, processed)
    }
}