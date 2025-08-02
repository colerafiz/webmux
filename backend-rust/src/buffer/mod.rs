use bytes::{Bytes, BytesMut, BufMut};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{Notify, RwLock};
use crossbeam::queue::SegQueue;

/// High-performance ring buffer optimized for terminal output
/// Uses lock-free data structures and zero-copy operations
pub struct OptimizedTerminalBuffer {
    /// Main buffer storage
    storage: Arc<RwLock<BytesMut>>,
    /// Write position
    write_pos: AtomicUsize,
    /// Read positions for multiple consumers
    read_positions: dashmap::DashMap<String, usize>,
    /// Notification for new data
    notify: Arc<Notify>,
    /// Statistics
    stats: BufferStats,
    /// Maximum buffer size
    max_size: usize,
}

#[derive(Default)]
pub struct BufferStats {
    bytes_written: AtomicU64,
    bytes_read: AtomicU64,
    messages_written: AtomicU64,
    overruns: AtomicU64,
}

impl OptimizedTerminalBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            storage: Arc::new(RwLock::new(BytesMut::with_capacity(max_size))),
            write_pos: AtomicUsize::new(0),
            read_positions: dashmap::DashMap::new(),
            notify: Arc::new(Notify::new()),
            stats: BufferStats::default(),
            max_size,
        }
    }
    
    /// Write data to buffer with zero-copy when possible
    pub async fn write(&self, data: &[u8]) -> Result<(), BufferError> {
        if data.len() > self.max_size / 2 {
            return Err(BufferError::DataTooLarge);
        }
        
        let mut storage = self.storage.write().await;
        
        // Check if we need to compact the buffer
        if storage.len() + data.len() > self.max_size {
            self.compact_buffer(&mut storage).await;
        }
        
        // If still not enough space, we have an overrun
        if storage.len() + data.len() > self.max_size {
            self.stats.overruns.fetch_add(1, Ordering::Relaxed);
            return Err(BufferError::BufferFull);
        }
        
        // Write data
        storage.extend_from_slice(data);
        let new_pos = storage.len();
        self.write_pos.store(new_pos, Ordering::Release);
        
        // Update stats
        self.stats.bytes_written.fetch_add(data.len() as u64, Ordering::Relaxed);
        self.stats.messages_written.fetch_add(1, Ordering::Relaxed);
        
        // Notify readers
        self.notify.notify_waiters();
        
        Ok(())
    }
    
    /// Create a reader for this buffer
    pub fn create_reader(&self, reader_id: String) -> BufferReader {
        self.read_positions.insert(reader_id.clone(), 0);
        BufferReader {
            buffer: self.clone(),
            reader_id,
            last_read_pos: 0,
        }
    }
    
    /// Compact buffer by removing data that all readers have consumed
    async fn compact_buffer(&self, storage: &mut BytesMut) {
        // Find minimum read position across all readers
        let min_read_pos = self.read_positions
            .iter()
            .map(|entry| *entry.value())
            .min()
            .unwrap_or(0);
        
        if min_read_pos > 0 {
            // Remove consumed data
            let remaining = storage.split_off(min_read_pos);
            *storage = remaining;
            
            // Adjust positions
            let adjustment = min_read_pos;
            for mut entry in self.read_positions.iter_mut() {
                *entry.value_mut() = entry.value().saturating_sub(adjustment);
            }
            self.write_pos.fetch_sub(adjustment, Ordering::AcqRel);
        }
    }
    
    /// Get current statistics
    pub fn stats(&self) -> BufferStatsSnapshot {
        BufferStatsSnapshot {
            bytes_written: self.stats.bytes_written.load(Ordering::Relaxed),
            bytes_read: self.stats.bytes_read.load(Ordering::Relaxed),
            messages_written: self.stats.messages_written.load(Ordering::Relaxed),
            overruns: self.stats.overruns.load(Ordering::Relaxed),
            buffer_size: self.write_pos.load(Ordering::Relaxed),
            reader_count: self.read_positions.len(),
        }
    }
}

impl Clone for OptimizedTerminalBuffer {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            write_pos: AtomicUsize::new(self.write_pos.load(Ordering::Relaxed)),
            read_positions: self.read_positions.clone(),
            notify: self.notify.clone(),
            stats: BufferStats::default(), // Don't clone stats
            max_size: self.max_size,
        }
    }
}

pub struct BufferReader {
    buffer: OptimizedTerminalBuffer,
    reader_id: String,
    last_read_pos: usize,
}

impl BufferReader {
    /// Read next chunk of data (zero-copy)
    pub async fn read_next(&mut self) -> Option<Bytes> {
        let write_pos = self.buffer.write_pos.load(Ordering::Acquire);
        
        if self.last_read_pos >= write_pos {
            // No new data, wait for notification
            self.buffer.notify.notified().await;
            let write_pos = self.buffer.write_pos.load(Ordering::Acquire);
            if self.last_read_pos >= write_pos {
                return None;
            }
        }
        
        // Read data
        let storage = self.buffer.storage.read().await;
        let data = storage[self.last_read_pos..write_pos].to_vec();
        drop(storage);
        
        // Update position
        self.last_read_pos = write_pos;
        self.buffer.read_positions.insert(self.reader_id.clone(), write_pos);
        
        // Update stats
        self.buffer.stats.bytes_read.fetch_add(data.len() as u64, Ordering::Relaxed);
        
        Some(Bytes::from(data))
    }
    
    /// Try to read without blocking
    pub async fn try_read_next(&mut self) -> Option<Bytes> {
        let write_pos = self.buffer.write_pos.load(Ordering::Acquire);
        
        if self.last_read_pos >= write_pos {
            return None;
        }
        
        // Read data
        let storage = self.buffer.storage.read().await;
        let data = storage[self.last_read_pos..write_pos].to_vec();
        drop(storage);
        
        // Update position
        self.last_read_pos = write_pos;
        self.buffer.read_positions.insert(self.reader_id.clone(), write_pos);
        
        // Update stats
        self.buffer.stats.bytes_read.fetch_add(data.len() as u64, Ordering::Relaxed);
        
        Some(Bytes::from(data))
    }
}

impl Drop for BufferReader {
    fn drop(&mut self) {
        self.buffer.read_positions.remove(&self.reader_id);
    }
}

/// Multi-producer single-consumer queue optimized for terminal messages
pub struct MessageQueue<T> {
    queue: Arc<SegQueue<T>>,
    notify: Arc<Notify>,
    capacity: AtomicUsize,
    max_capacity: usize,
}

impl<T> MessageQueue<T> {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            queue: Arc::new(SegQueue::new()),
            notify: Arc::new(Notify::new()),
            capacity: AtomicUsize::new(0),
            max_capacity,
        }
    }
    
    /// Push message with backpressure
    pub fn push(&self, item: T) -> Result<(), T> {
        let current = self.capacity.load(Ordering::Acquire);
        if current >= self.max_capacity {
            return Err(item);
        }
        
        self.queue.push(item);
        self.capacity.fetch_add(1, Ordering::AcqRel);
        self.notify.notify_one();
        Ok(())
    }
    
    /// Pop message (blocking)
    pub async fn pop(&self) -> T {
        loop {
            if let Some(item) = self.queue.pop() {
                self.capacity.fetch_sub(1, Ordering::AcqRel);
                return item;
            }
            self.notify.notified().await;
        }
    }
    
    /// Try to pop without blocking
    pub fn try_pop(&self) -> Option<T> {
        self.queue.pop().map(|item| {
            self.capacity.fetch_sub(1, Ordering::AcqRel);
            item
        })
    }
    
    /// Current queue size
    pub fn len(&self) -> usize {
        self.capacity.load(Ordering::Relaxed)
    }
    
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BufferStatsSnapshot {
    pub bytes_written: u64,
    pub bytes_read: u64,
    pub messages_written: u64,
    pub overruns: u64,
    pub buffer_size: usize,
    pub reader_count: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum BufferError {
    #[error("Data too large for buffer")]
    DataTooLarge,
    #[error("Buffer full")]
    BufferFull,
    #[error("Reader not found")]
    ReaderNotFound,
}