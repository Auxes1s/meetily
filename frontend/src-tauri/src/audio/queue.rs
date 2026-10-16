//! Nonblocking, byte-budgeted PCM handoff. Inference backlog belongs on disk.
use super::recording_state::{AudioChunk, TranscriptionChunk};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
static MEMORY_BYTES: AtomicU64 = AtomicU64::new(0);
pub fn memory_bytes() -> u64 {
    MEMORY_BYTES.load(Ordering::SeqCst)
}
use tokio::sync::{mpsc, OwnedSemaphorePermit, Semaphore};
pub const PCM_BUDGET: usize = 8 * 1024 * 1024;
pub trait PcmPayload {
    fn pcm_bytes(&self) -> usize;
}
impl PcmPayload for AudioChunk {
    fn pcm_bytes(&self) -> usize {
        self.data.len() * 4
    }
}
impl PcmPayload for TranscriptionChunk {
    fn pcm_bytes(&self) -> usize {
        self.audio.pcm_bytes()
    }
}
struct Reservation {
    _permit: OwnedSemaphorePermit,
    bytes: u64,
}
impl Drop for Reservation {
    fn drop(&mut self) {
        MEMORY_BYTES.fetch_sub(self.bytes, Ordering::SeqCst);
    }
}
struct Envelope<T> {
    value: T,
    _permit: Reservation,
}
pub struct PcmSender<T> {
    sender: mpsc::UnboundedSender<Envelope<T>>,
    budget: Arc<Semaphore>,
    error: Arc<Mutex<Option<String>>>,
}
pub struct PcmReceiver<T> {
    receiver: mpsc::UnboundedReceiver<Envelope<T>>,
    error: Arc<Mutex<Option<String>>>,
}
impl<T> Clone for PcmSender<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            budget: self.budget.clone(),
            error: self.error.clone(),
        }
    }
}
pub fn pcm_channel<T>() -> (PcmSender<T>, PcmReceiver<T>) {
    channel_with_budget(PCM_BUDGET)
}
pub fn channel_with_budget<T>(bytes: usize) -> (PcmSender<T>, PcmReceiver<T>) {
    let (sender, receiver) = mpsc::unbounded_channel();
    let error = Arc::new(Mutex::new(None));
    (
        PcmSender {
            sender,
            budget: Arc::new(Semaphore::new(bytes)),
            error: error.clone(),
        },
        PcmReceiver { receiver, error },
    )
}
impl<T: PcmPayload> PcmSender<T> {
    pub fn send(&self, value: T) -> anyhow::Result<()> {
        let size = value.pcm_bytes().max(1);
        let permit = u32::try_from(size)
            .ok()
            .and_then(|n| self.budget.clone().try_acquire_many_owned(n).ok());
        let result = match permit {
            Some(permit) => { MEMORY_BYTES.fetch_add(size as u64,Ordering::SeqCst); self.sender.send(Envelope { value, _permit: Reservation { _permit:permit,bytes:size as u64 } }).map_err(|_| anyhow::anyhow!("PCM consumer stopped")) },
            None => Err(anyhow::anyhow!("PCM intake exceeded its byte budget; recording is interrupted, retained audio is recoverable")),
        };
        if let Err(error) = &result {
            *self.error.lock().unwrap() = Some(error.to_string());
        }
        result
    }
}
impl<T> PcmReceiver<T> {
    pub async fn recv(&mut self) -> Option<T> {
        self.receiver.recv().await.map(|e| e.value)
    }
    pub fn blocking_recv(&mut self) -> Option<T> {
        self.receiver.blocking_recv().map(|e| e.value)
    }
    pub fn try_recv(&mut self) -> Result<T, mpsc::error::TryRecvError> {
        self.receiver.try_recv().map(|e| e.value)
    }
    pub fn is_empty(&self) -> bool {
        self.receiver.is_empty()
    }
    pub fn error(&self) -> Option<String> {
        self.error.lock().unwrap().clone()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn chunk() -> AudioChunk {
        AudioChunk {
            data: vec![0.; 4],
            sample_rate: 16000,
            timestamp: 0.,
            chunk_id: 0,
            device_type: super::super::recording_state::DeviceType::Microphone,
        }
    }
    #[tokio::test]
    async fn byte_budget_releases_on_receive_and_overflow_is_observable() {
        let (tx, mut rx) = channel_with_budget(16);
        tx.send(chunk()).unwrap();
        assert!(tx.send(chunk()).is_err());
        assert_eq!(rx.recv().await.unwrap().data.len(), 4);
        tx.send(chunk()).unwrap();
        assert!(rx.error().is_some());
        drop(tx);
        assert!(rx.recv().await.is_some());
        assert!(rx.recv().await.is_none());
    }
}
