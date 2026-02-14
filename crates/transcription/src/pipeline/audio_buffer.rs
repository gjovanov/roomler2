use std::collections::VecDeque;

/// Ring buffer that holds the last N audio frames for pre-speech padding.
///
/// When speech is detected, the contents of this buffer are prepended to
/// the speech audio to avoid cutting off the beginning of utterances.
pub struct AudioRingBuffer {
    buffer: VecDeque<Vec<f32>>,
    max_frames: usize,
}

impl AudioRingBuffer {
    pub fn new(max_frames: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(max_frames + 1),
            max_frames,
        }
    }

    /// Pushes a frame into the ring buffer, evicting the oldest if full.
    pub fn push(&mut self, frame: Vec<f32>) {
        if self.buffer.len() >= self.max_frames {
            self.buffer.pop_front();
        }
        self.buffer.push_back(frame);
    }

    /// Drains all frames and returns them concatenated into a single buffer.
    pub fn drain_all(&mut self) -> Vec<f32> {
        let total: usize = self.buffer.iter().map(|f| f.len()).sum();
        let mut result = Vec::with_capacity(total);
        for frame in self.buffer.drain(..) {
            result.extend(frame);
        }
        result
    }

    /// Returns the total number of samples across all buffered frames.
    pub fn total_samples(&self) -> usize {
        self.buffer.iter().map(|f| f.len()).sum()
    }

    /// Clears the buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_drain() {
        let mut buf = AudioRingBuffer::new(3);
        buf.push(vec![1.0, 2.0]);
        buf.push(vec![3.0, 4.0]);
        buf.push(vec![5.0, 6.0]);
        let drained = buf.drain_all();
        assert_eq!(drained, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn evicts_oldest() {
        let mut buf = AudioRingBuffer::new(2);
        buf.push(vec![1.0]);
        buf.push(vec![2.0]);
        buf.push(vec![3.0]); // evicts [1.0]
        let drained = buf.drain_all();
        assert_eq!(drained, vec![2.0, 3.0]);
    }

    #[test]
    fn total_samples() {
        let mut buf = AudioRingBuffer::new(3);
        buf.push(vec![1.0, 2.0, 3.0]);
        buf.push(vec![4.0, 5.0]);
        assert_eq!(buf.total_samples(), 5);
    }
}
