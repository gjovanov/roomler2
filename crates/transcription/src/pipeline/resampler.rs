use rubato::{
    Async as AsyncResampler, FixedAsync, Resampler as RubatoResampler,
    SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use audioadapter_buffers::direct::InterleavedSlice;

/// Resamples audio from 48kHz mono to 16kHz mono using sinc interpolation.
pub struct Resampler {
    inner: AsyncResampler<f32>,
    /// Accumulator for input samples that don't fill a complete chunk.
    pending: Vec<f32>,
    /// Number of input frames the resampler expects per process() call.
    chunk_size: usize,
}

impl Resampler {
    /// Creates a new 48kHz -> 16kHz mono resampler.
    ///
    /// `chunk_size` is the number of input frames per resampling call (e.g. 960 = 20ms at 48kHz).
    pub fn new(chunk_size: usize) -> anyhow::Result<Self> {
        let params = SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Linear,
            oversampling_factor: 256,
            window: WindowFunction::BlackmanHarris2,
        };

        let inner = AsyncResampler::<f32>::new_sinc(
            16000.0 / 48000.0, // resample ratio
            2.0,               // max relative ratio
            &params,
            chunk_size,
            1,                 // mono channel
            FixedAsync::Input, // fixed input size
        )
        .map_err(|e| anyhow::anyhow!("Failed to create resampler: {}", e))?;

        Ok(Self {
            inner,
            pending: Vec::with_capacity(chunk_size * 2),
            chunk_size,
        })
    }

    /// Feeds mono 48kHz samples and returns resampled 16kHz samples.
    ///
    /// Buffers internally if input doesn't fill a complete resampler chunk.
    /// Returns an empty vec if not enough data yet.
    pub fn process(&mut self, input: &[f32]) -> anyhow::Result<Vec<f32>> {
        self.pending.extend_from_slice(input);

        let mut output = Vec::new();

        while self.pending.len() >= self.chunk_size {
            let chunk: Vec<f32> = self.pending.drain(..self.chunk_size).collect();
            let frames = chunk.len(); // mono: 1 sample = 1 frame
            let input_adapter = InterleavedSlice::new(&chunk, 1, frames)
                .map_err(|e| anyhow::anyhow!("Input adapter error: {}", e))?;

            let resampled = self
                .inner
                .process(&input_adapter, 0, None)
                .map_err(|e| anyhow::anyhow!("Resample error: {}", e))?;

            // InterleavedOwned stores interleaved samples; for mono, just take the data
            output.extend(resampled.take_data());
        }

        Ok(output)
    }

    /// Flushes any remaining buffered samples (with zero-padding).
    pub fn flush(&mut self) -> anyhow::Result<Vec<f32>> {
        if self.pending.is_empty() {
            return Ok(Vec::new());
        }

        // Pad to chunk size
        self.pending.resize(self.chunk_size, 0.0);
        self.process(&[])
    }
}
