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
    ///Return a lfo with saw as the default waveform
    pub fn build_lfo(frequence: f32, sample_rate: f32) -> Self {
        Lfo {
            frequence,
            waveform: Waveform::Saw,
            phasor: 0.,
            sample_rate,
        }
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

pub struct HarmonicOscillator {
    //Parameter
    waveform: Waveform,
    tune: f32,
    frequency_hz: f32,
    lfo_frequency: f32,
    number_of_periods: f32,
    //computation variable
    sample_rate: f32,
    current_sample_index: f32,
    lfo_current_sample_index: f32,
    number_of_harmonics: f32,
    modulation: f32,
}

impl HarmonicOscillator {
    pub fn new(sample_rate: f32, frequency_hz: f32, lfo_frequency: f32) -> Self {
        HarmonicOscillator {
            sample_rate,
            frequency_hz,
            lfo_frequency,
            number_of_periods: 1.,
            waveform: Waveform::Saw,
            tune: 0.,
            current_sample_index: 0.,
            lfo_current_sample_index: 0.,
            number_of_harmonics: 1.,
            modulation: 0.,
        }
    }

    pub fn set_note(&mut self, midi_note: u8) {
        self.frequency_hz = midi_to_frequence(midi_note);
    }

    pub fn set_parameters(
        &mut self,
        tune: f32,
        waveform: Waveform,
        lfo_frequency: f32,
        number_of_periods: f32,
    ) {
        self.tune = tune;
        self.lfo_frequency = lfo_frequency;
        self.waveform = waveform;
        self.number_of_periods = number_of_periods;
    }

    fn advance_sample(&mut self) {
        self.current_sample_index = (self.current_sample_index + 1.0) % self.sample_rate;
        self.lfo_current_sample_index =
            (self.lfo_current_sample_index + (self.lfo_frequency / self.sample_rate)) % 1.0;
    }

    fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    fn get_lfo_gain(&self, mut index: f32) -> f32 {
        index = index / self.number_of_harmonics;
        index = (self.lfo_current_sample_index + index) % (1. / self.number_of_periods);
        index *= self.number_of_periods;
        index
    }

    fn calculate_sine_output_from_freq(&self, freq: f32) -> f32 {
        let two_pi = 2.0 * std::f32::consts::PI;
        (self.current_sample_index * freq * two_pi / self.sample_rate).sin()
    }

    fn is_multiple_of_freq_above_nyquist(&self, multiple: f32) -> bool {
        self.frequency_hz * multiple > self.sample_rate / 2.0
    }

    fn sine_wave(&mut self) -> f32 {
        self.advance_sample();
        self.calculate_sine_output_from_freq(self.frequency_hz)
    }

    fn generative_waveform(&mut self, harmonic_index_increment: i32, gain_exponent: f32) -> f32 {
        self.advance_sample();
        let mut output = 0.0;
        let mut i = 1;
        let mut number_of_harmonics = 1.0;
        while !self.is_multiple_of_freq_above_nyquist(i as f32) {
            let gain = 1.0 / (i as f32).powf(gain_exponent);
            let lfo_gain = self.get_lfo_gain(number_of_harmonics);
            let sine = self.calculate_sine_output_from_freq(
                (self.frequency_hz + self.modulation).clamp(20., 20000.) * (i as f32),
            );
            output += gain * lfo_gain * sine;
            // * self.get_lfo_gain(number_of_harmonics)
            // * self.calculate_sine_output_from_freq(
            //     (self.frequency_hz + self.modulation).clamp(20., 20000.) * (i as f32),
            // );
            i += harmonic_index_increment;
            number_of_harmonics += 1.;
        }
        self.number_of_harmonics = number_of_harmonics;
        output
    }

    fn square_wave(&mut self) -> f32 {
        self.generative_waveform(2, 1.0)
    }

    fn saw_wave(&mut self) -> f32 {
        self.generative_waveform(1, 1.0)
    }

    fn triangle_wave(&mut self) -> f32 {
        self.generative_waveform(2, 2.0)
    }

    pub fn tick(&mut self) -> f32 {
        match self.waveform {
            Waveform::Sine => self.sine_wave(),
            Waveform::Square => self.square_wave(),
            Waveform::Saw => self.saw_wave(),
            Waveform::Triangle => self.triangle_wave(),
        }
    }

    pub fn modulate(&mut self, modulation: f32) {
        self.modulation = modulation;
    }
}
