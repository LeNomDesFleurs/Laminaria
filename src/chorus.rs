use crate::Lfo;
use crate::RingBuffer;
pub struct Chorus {
    lfo: Lfo,
    buffer: RingBuffer,
    amplitude: f32,
}

impl Chorus {
    pub fn new(sample_rate: f32) -> Self {
        Chorus {
            lfo: Lfo::build_lfo(0.2, sample_rate),
            buffer: RingBuffer::new(sample_rate, 0.050),
            amplitude: 0.1,
        }
    }

    pub fn set_parameters(&mut self, amplitude: f32, rate: f32) {
        self.lfo.frequence = rate;
        self.amplitude = amplitude;
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        self.buffer
            .set_step_size((self.lfo.tick() * self.amplitude) + 1.);
        self.buffer.write_sample(sample);
        (self.buffer.read_sample() + sample) / 2.
    }
}
