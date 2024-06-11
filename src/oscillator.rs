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
            waveform: Waveform::Triangle,
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
    
#[derive(Clone, Copy)]
pub struct HarmonicOscillator {
    //Parameter
    pub frequency_hz: f32,
    pub lfo_frequency: f32,
    pub number_of_periods: f32,
    //computation variable
    sample_rate: f32,
    pub current_sample_index: f32,
    lfo_current_sample_index: f32,
    number_of_harmonics: f32,
    modulation: f32,
    //waveform description, sqr is 2 / 1, tri is 2 / 2
    pub harmonic_index_increment: f32,
    pub harmonic_gain_exponent: f32,
}

impl HarmonicOscillator {
    pub fn new(sample_rate: f32, frequency_hz: f32, lfo_frequency: f32) -> Self {
        HarmonicOscillator {
            sample_rate,
            frequency_hz,
            lfo_frequency,
            number_of_periods: 1.,
            current_sample_index: 0.,
            lfo_current_sample_index: 0.,
            number_of_harmonics: 1.,
            modulation: 0.,
            harmonic_gain_exponent: 1.,
            harmonic_index_increment: 1.,
        }
    }

    pub fn set_note(&mut self, midi_note: u8) {
        self.frequency_hz = midi_to_frequence(midi_note);
    }

    fn advance_sample(&mut self) {
        self.current_sample_index = (self.current_sample_index + 1.0) % self.sample_rate;
        self.lfo_current_sample_index =
            (self.lfo_current_sample_index + (self.lfo_frequency / self.sample_rate)) % 1.0;
    }

    fn get_lfo_gain(&self, mut index: f32) -> f32 {
        index = index / self.number_of_harmonics;
        index = (self.lfo_current_sample_index + index) % (1. / self.number_of_periods);
        // Get tri out of saw
        index -= 0.5 ;
        index = index.abs();
        index *= 2.;
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

    pub fn process(&mut self) -> f32 {
        self.advance_sample();
        let mut output = 0.0;
        let mut i = 1.;
        let mut number_of_harmonics = 1.0;
        while !self.is_multiple_of_freq_above_nyquist(i as f32) || number_of_harmonics<20. {
            let gain = 1.0 / (i as f32).powf(self.harmonic_gain_exponent);
            let lfo_gain = 1.;
            // self.get_lfo_gain(number_of_harmonics);
            let sine = self.calculate_sine_output_from_freq(
                (self.frequency_hz).clamp(20., 20000.) * (i as f32),
            );
            output += gain * lfo_gain * sine;
            // * self.get_lfo_gain(number_of_harmonics)
            // * self.calculate_sine_output_from_freq(
            //     (self.frequency_hz + self.modulation).clamp(20., 20000.) * (i as f32),
            // );
            i += self.harmonic_index_increment;
            number_of_harmonics += 1.;
        }
        self.number_of_harmonics = number_of_harmonics;
        //volume adjustement
        output *= ((self.harmonic_gain_exponent-0.01)/3.0).powf(1.2) * 100.;
        output /= 100.;
        output *= (self.harmonic_index_increment/2.0).powf(1.0);
        output
    }

 
}
