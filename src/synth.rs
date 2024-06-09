use std::collections::HashMap;

use crate::envelope::Envelope;
use crate::filter::FilterType;
use crate::midi;
use crate::oscillator::Waveform;
use crate::outils::midi_to_frequence;
use crate::parameters;
use crate::parameters::get_parameters;
use crate::Biquad;
use crate::Chorus;
use crate::HarmonicOscillator;
use crate::Lfo;
use crate::RingBuffer;
use crate::TextCharacteristic;

pub struct Synth {
    current_note: Option<u8>,
    envelope: Envelope,
    filter: Biquad,
    oscillator: HarmonicOscillator,
    lfo: Lfo,
    chorus: Chorus,
    buffer: RingBuffer,
    pub parameters: HashMap<String, parameters::Parameter>,
    // parameters: Parameters,
    routing_delay_time: f32,
    osc_to_filter_amp: f32,
    lfo_to_osc: f32,
}

pub struct Parameters {
    filter_freq: f32,

    osc_freq: f32,
    osc_shape: Waveform,
    osc_mod_freq: f32,
    osc_mod_period: f32,

    chorus_rate: f32,
    chorus_amp: f32,

    lfo_freq: f32,
    lfo_shape: Waveform,
}

impl Synth {
    pub fn default(sample_rate: f32) -> Self {
        Synth {
            current_note: None,
            filter: Biquad::default(sample_rate, FilterType::LPF, 1000., 0.707),
            envelope: Envelope::new(sample_rate as i32),
            lfo: Lfo::build_lfo(0.2, sample_rate),
            oscillator: HarmonicOscillator::new(sample_rate, 500., 0.2),
            buffer: RingBuffer::default(sample_rate, 0.05),
            chorus: Chorus::default(sample_rate),
            routing_delay_time: 0.5,
            osc_to_filter_amp: 0.1,
            lfo_to_osc: 0.1,
            parameters: get_parameters(),
            // parameters: Parameters {},
        }
    }
    pub fn set_note(&mut self, midi_note: u8, note_on: bool) {
        match self.current_note {
            //if not playing, start a new note
            None => {
                if note_on {
                    self.current_note = Some(midi_note);
                    self.envelope.note_statut(true);
                };
            }
            // if playing, end the note if is the correct noteOff, or change to a new one it is a noteOn
            Some(current_note) => match note_on {
                false => {
                    if midi_note == current_note {
                        self.current_note = None;
                        self.envelope.note_statut(false);
                    }
                }
                true => {
                    if midi_note != current_note {
                        self.current_note = Some(midi_note);
                    }
                }
            },
        }
        if self.current_note != None {
            self.oscillator.set_note(self.current_note.clone().unwrap())
        }
    }

    pub fn mapping(&mut self, text_characteristic: TextCharacteristic) {
        let vowel = text_characteristic.number_of_vowel as f32;
        let consonant = text_characteristic.number_of_consonant as f32;
        let space = text_characteristic.number_of_space as f32;
        let special = text_characteristic.number_of_special_character as f32;

        let parameters = Parameters {
            filter_freq: vowel * 200.,
            osc_freq: space * 100. + 100.,
            osc_shape: match space as i32 {
                0 => Waveform::Saw,
                1 => Waveform::Triangle,
                2 => Waveform::Square,
                _ => Waveform::Saw,
            },
            osc_mod_freq: space / consonant,
            osc_mod_period: vowel / (consonant + 1.),
            chorus_rate: special / 2.,
            chorus_amp: (consonant / (vowel + 1.)) / 10.,
            lfo_freq: special / 0.7,
            lfo_shape: match special as i32 {
                0 => Waveform::Saw,
                1 => Waveform::Triangle,
                2 => Waveform::Square,
                _ => Waveform::Saw,
            },
        };

        self.osc_to_filter_amp = (consonant * 10.) as f32;
        self.routing_delay_time = 0.30;
        self.lfo_to_osc = (consonant / (vowel + 1.)) * 4.;

        self.set_parameters(parameters)
    }

    pub fn tick(&mut self) -> f32 {
        self.set_parameters2();
        self.filter
            .modulate(self.buffer.read_sample() * self.osc_to_filter_amp);
        self.oscillator.modulate(self.lfo.tick() * self.lfo_to_osc);
        let mut sample = self.oscillator.tick();
        //vca
        let envelope = self.envelope.process();
        sample *= envelope; 
        // EFFECTS
        self.buffer.write_sample(sample);
        sample = self.filter.process(sample);
        sample = self.chorus.process(sample);
        return sample;
    }

    pub fn set_parameters(&mut self, parameters: Parameters) {
        self.filter.set_frequency(parameters.filter_freq);
        self.oscillator.set_parameters(
            parameters.osc_freq,
            parameters.osc_shape,
            parameters.osc_mod_freq,
            parameters.osc_mod_period,
        );
        self.buffer.set_delay_time(self.routing_delay_time);
        self.chorus
            .set_parameters(parameters.chorus_amp, parameters.chorus_rate);
        self.lfo
            .set_freq_and_shape(parameters.lfo_freq, parameters.lfo_shape);
    }

    pub fn set_parameters2(&mut self) {
        self.filter
            .set_frequency(self.parameters["fil-freq"].get_raw_value());
        self.oscillator.set_parameters(
            self.parameters["osc-tune"].get_raw_value(),
            Waveform::Sine,
            self.parameters["lfo-freq"].get_raw_value(),
            self.parameters["lfo-period"].get_raw_value(),
        );
        self.buffer.set_delay_time(self.routing_delay_time);
        self.chorus.set_parameters(0.5, 0.1);
        self.lfo.set_freq_and_shape(
            self.parameters["lfo-freq"].get_raw_value(),
            Waveform::Square,
        );
        self.envelope.set_attack(self.parameters["env-atk"].get_raw_value());
        self.envelope.set_release(self.parameters["env-dcy"].get_raw_value());
        
    }
}
