use crate::outils::midi_to_frequence;

pub enum Waveform {
    Sine,
    Square,
    Saw,
    Triangle,
}

pub struct Lfo {
    pub frequence: f32,
    waveform: Waveform,
    phasor: f32,
    sample_rate: f32,
}

impl Lfo {
    pub fn new(frequence: f32) -> Self {
        Lfo {
            frequence,
            waveform: Waveform::Triangle,
            phasor: 0.,
            sample_rate: 0.,
        }
    }

    ///Return a lfo with saw as the default waveform
    pub fn init(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    pub fn set_freq_and_shape(&mut self, frequence: f32, waveform: Waveform) {
        self.frequence = frequence;
        self.waveform = waveform;
    }

    pub fn set_frequence(&mut self, frequence: f32) {
        self.frequence = frequence;
    }

    fn increment_phasor(&mut self) {
        self.phasor = (self.phasor + (self.frequence / self.sample_rate)) % 1.;
    }

    fn sine(&self) -> f32 {
        let two_pi = 2.0 * std::f32::consts::PI;
        (self.phasor * two_pi).sin()
    }

    fn saw(&self) -> f32 {
        let mut temp = self.phasor;
        temp -= 0.5;
        temp *= 2.;
        temp
    }

    fn square(&self) -> f32 {
        let mut temp = 0.;
        if self.phasor > 0.5 {
            temp = 1.
        }
        temp -= 0.5;
        temp *= 2.;
        temp
    }

    fn triangle(&self) -> f32 {
        let mut temp = self.phasor;
        temp = temp - 0.5;
        temp = temp.abs();
        temp = temp * 4.;
        temp = temp - 1.;
        temp
    }

    pub fn tick(&mut self) -> f32 {
        self.increment_phasor();
        match self.waveform {
            Waveform::Square => self.square(),
            Waveform::Sine => self.sine(),
            Waveform::Saw => self.saw(),
            Waveform::Triangle => self.triangle(),
            // _ => 0.,
        }
    }
}

#[derive(Clone, Copy)]
pub struct SineWave {
    pub frequency_hz: f32,
    pub sample_rate: f32,
    pub phasor: f32,
}

impl SineWave {
    pub fn new() -> Self {
        Self {
            frequency_hz: 440.0,
            sample_rate: 0.0,
            phasor: 0.0,
        }
    }

    pub fn init(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    pub fn set_note(&mut self, midi_note: u8) {
        self.frequency_hz = midi_to_frequence(midi_note);
    }

    fn increment_phasor(&mut self) {
        self.phasor = (self.phasor + (self.frequency_hz / self.sample_rate)) % 1.;
    }

    fn sine(&self) -> f32 {
        let two_pi = 2.0 * std::f32::consts::PI;
        (self.phasor * two_pi).sin()
    }

    pub fn process(&mut self) -> f32 {
        self.increment_phasor();
        self.sine()
    }
}

#[derive(Clone, Copy)]
pub struct HarmonicOscillator {
    //Parameter
    pub frequency_hz: f32,
    //computation variable
    sample_rate: f32,
    sine_bank: [SineWave; 5],
    pub current_sample_index: f32,
    //waveform description, sqr is 2 / 1, tri is 2 / 2
    pub harmonic_index_increment: f32,
    pub harmonic_gain_exponent: f32,
}

impl HarmonicOscillator {
    pub fn new(frequency_hz: f32) -> Self {
        HarmonicOscillator {
            sample_rate: 0.0,
            frequency_hz,
            sine_bank: [SineWave::new(); 5],
            current_sample_index: 0.0,
            harmonic_gain_exponent: 1.0,
            harmonic_index_increment: 1.0,
        }
    }

    pub fn init(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.sine_bank.iter_mut().for_each(|x| x.init(sample_rate));
    }

    pub fn set_note(&mut self, midi_note: u8) {
        self.frequency_hz = midi_to_frequence(midi_note);
    }

    pub fn process(&mut self) -> f32 {
        let mut output = 0.0;
        let mut i = 1.;

        for sine in self.sine_bank.iter_mut() {
            let gain = 1.0 / (i as f32).powf(self.harmonic_gain_exponent);
            sine.frequency_hz = self.frequency_hz * i;
            output += sine.process() * gain;
            i += self.harmonic_index_increment;
        }
        output /= 5.0;

        //volume adjustement
        // output *= ((self.harmonic_gain_exponent-0.01)/3.0).powf(1.2) * 100.;
        // output /= 100.;
        output *= (self.harmonic_index_increment / 2.0).powf(1.0);
        output
    }
}

pub struct Oscillator {
    pub sample_rate: f32,
    // pub waveform: Waveform,
    pub current_sample_index: f32,
    pub frequency_hz: f32,
    pub harmonic_index_increment: f32,
    pub gain_exponent: f32,
}

impl Oscillator {
    pub fn new() -> Self {
        Oscillator {
            sample_rate: 0.0,
            current_sample_index: 0.0,
            harmonic_index_increment: 1.0,
            gain_exponent: 1.0,
            frequency_hz: 500.0,
        }
    }

    pub fn init(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    pub fn set_note(&mut self, midi_note: u8) {
        self.frequency_hz = midi_to_frequence(midi_note);
    }

    fn advance_sample(&mut self) {
        self.current_sample_index = (self.current_sample_index + 1.0) % self.sample_rate;
    }

    // fn set_waveform(&mut self, waveform: Waveform) {
    //     self.waveform = waveform;
    // }

    fn calculate_sine_output_from_freq(&self, freq: f32) -> f32 {
        let two_pi = 2.0 * std::f32::consts::PI;
        (self.current_sample_index * freq * two_pi / self.sample_rate).sin()
    }

    fn is_multiple_of_freq_above_nyquist(&self, multiple: f32) -> bool {
        false
    }

    fn sine_wave(&mut self) -> f32 {
        self.advance_sample();
        self.calculate_sine_output_from_freq(self.frequency_hz)
    }

    fn process2(&mut self, harmonic_index_increment: f32, gain_exponent: f32) -> f32 {
        self.advance_sample();
        let mut output = 0.0;
        let mut i = 1.;
        while !self.is_multiple_of_freq_above_nyquist(i) {
            let gain = 1.0 / (i as f32).powf(gain_exponent);
            output += gain * self.calculate_sine_output_from_freq(self.frequency_hz * i);
            i += self.harmonic_index_increment;
        }
        output
    }

    pub fn process(&mut self) -> f32 {
        self.process2(1.73, 2.67)
    }
}
