use std::collections::HashMap;

use crate::buffer;
use crate::buffer::DelayLine;
use crate::envelope;
use crate::envelope::Envelope;
use crate::midibuffer::PolyMidiBuffer;
use crate::oscillator::Waveform;
use crate::outils;
use crate::parameters;
use crate::parameters::get_parameters;
use crate::Chorus;
use crate::HarmonicOscillator;
use crate::Lfo;
use crate::RingBuffer;
use crate::TextCharacteristic;

const NUMBER_OF_VOICES: usize = 4;
const VOICE_ITERATOR: std::ops::Range<usize> = 0..NUMBER_OF_VOICES;

pub struct Synth {
    envelopes: [Envelope; NUMBER_OF_VOICES],
    oscillators: [HarmonicOscillator; NUMBER_OF_VOICES],
    midibuffer: PolyMidiBuffer,
    lfo: Lfo,
    delay: DelayLine,
    chorus: Chorus,
    buffer: RingBuffer,
    pub parameters: HashMap<String, parameters::Parameter>,
    // parameters: Parameters,
    routing_delay_time: f32,
}

impl Synth {
    pub fn default(sample_rate: f32) -> Self {
        Synth {
            envelopes: [Envelope::new(sample_rate as i32); NUMBER_OF_VOICES],
            lfo: Lfo::build_lfo(0.2, sample_rate),
            oscillators: [HarmonicOscillator::new(sample_rate, 500., 0.2); NUMBER_OF_VOICES],
            midibuffer: PolyMidiBuffer::new(NUMBER_OF_VOICES),
            buffer: RingBuffer::new(sample_rate, 0.05),
            chorus: Chorus::new(sample_rate),
            routing_delay_time: 0.5,
            parameters: get_parameters(),
            delay: DelayLine::new(sample_rate, buffer::MAXIMUM_DELAY_TIME),
            // parameters: Parameters {},
        }
    }
    pub fn set_note(&mut self, midi_note: u8, note_on: bool) {
        if note_on {
            self.midibuffer.add_note(midi_note);
        } else {
            self.midibuffer.remove_note(midi_note);
        }

        for i in VOICE_ITERATOR {
            match self.midibuffer.notes.get(i) {
                None => {self.envelopes[i].note_off()},
                Some(midi_note) => { self.envelopes[i].note_on();
                    self.oscillators[i].set_note(*midi_note)            
                }
            }
        }
    }

    // pub fn mapping(&mut self, text_characteristic: TextCharacteristic) {
    //     let vowel = text_characteristic.number_of_vowel as f32;
    //     let consonant = text_characteristic.number_of_consonant as f32;
    //     let space = text_characteristic.number_of_space as f32;
    //     let special = text_characteristic.number_of_special_character as f32;

    //     let parameters = Parameters {
    //         filter_freq: vowel * 200.,
    //         osc_freq: space * 100. + 100.,
    //         osc_shape: match space as i32 {
    //             0 => Waveform::Saw,
    //             1 => Waveform::Triangle,
    //             2 => Waveform::Square,
    //             _ => Waveform::Saw,
    //         },
    //         osc_mod_freq: space / consonant,
    //         osc_mod_period: vowel / (consonant + 1.),
    //         chorus_rate: special / 2.,
    //         chorus_amp: (consonant / (vowel + 1.)) / 10.,
    //         lfo_freq: special / 0.7,
    //         lfo_shape: match special as i32 {
    //             0 => Waveform::Saw,
    //             1 => Waveform::Triangle,
    //             2 => Waveform::Square,
    //             _ => Waveform::Saw,
    //         },
    //     };

    //     self.osc_to_filter_amp = (consonant * 10.) as f32;
    //     self.routing_delay_time = 0.30;
    //     self.lfo_to_osc = (consonant / (vowel + 1.)) * 4.;

    //     self.set_parameters(parameters)
    // }

    pub fn process(&mut self) -> f32 {
        self.set_parameters();
        let mut sample: f32 = 0.;
        for i in VOICE_ITERATOR {
            match self.envelopes[i].status{
                envelope::Segment::Off => {}
                _ => {sample += self.oscillators[i].process() * self.envelopes[i].process()}
            }
        }
        sample /= 4.;
        //vca
        // EFFECTS
        sample = outils::equal_power_crossfade(
            sample,
            self.delay.process(sample),
            self.parameters["dly-wet"].get_raw_value(),
        );
        sample *= self.parameters["volume"].get_raw_value();
        return sample;
    }

    // pub fn set_parameters(&mut self, parameters: Parameters) {
    //     self.filter.set_frequency(parameters.filter_freq);
    //     self.oscillator.set_parameters(
    //         parameters.osc_freq,
    //         parameters.osc_shape,
    //         parameters.osc_mod_freq,
    //         parameters.osc_mod_period,
    //     );
    //     self.buffer.set_delay_time(self.routing_delay_time);
    //     self.chorus
    //         .set_parameters(parameters.chorus_amp, parameters.chorus_rate);
    //     self.lfo
    //         .set_freq_and_shape(parameters.lfo_freq, parameters.lfo_shape);
    // }

    pub fn set_parameters(&mut self) {
        //oscillator
        for i in VOICE_ITERATOR {
            self.oscillators[i].lfo_frequency = self.parameters["lfo-freq"].get_raw_value();
            self.oscillators[i].number_of_periods = self.parameters["lfo-period"].get_raw_value();
            self.oscillators[i].harmonic_gain_exponent =
                self.parameters["osc-hrmgn"].get_raw_value();
            self.oscillators[i].harmonic_index_increment =
                self.parameters["osc-hrmrat"].get_raw_value();
            self.envelopes[i].set_attack(self.parameters["env-atk"].get_raw_value());
            self.envelopes[i].set_release(self.parameters["env-dcy"].get_raw_value());
        }

        //delay (dry/wet implemented in process)
        self.delay
            .set_time(self.parameters["dly-time"].get_raw_value());
        self.delay
            .set_feedback(self.parameters["dly-feed"].get_raw_value());
    }
}
