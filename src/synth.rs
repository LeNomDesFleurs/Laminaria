use std::collections::HashMap;

use crate::buffer;
use crate::buffer::DelayLine;
use crate::envelope;
use crate::envelope::Envelope;
use crate::midibuffer::PolyMidiBuffer;
use crate::oscillator::Oscillator;
use crate::oscillator::Waveform;
use crate::outils;
use crate::parameters;
use crate::parameters::Parameter;
use crate::parameters::ParameterCapsule;
use crate::parameters::ParameterID;
use crate::parameters::Parameters;
use crate::Biquad;
use crate::Chorus;
use crate::HarmonicOscillator;
use crate::Lfo;
use crate::RingBuffer;
use crate::TextCharacteristic;
type ID = ParameterID;
use crate::ParameterUpdate;

const NUMBER_OF_VOICES: usize = 4;
const VOICE_ITERATOR: std::ops::Range<usize> = 0..NUMBER_OF_VOICES;

pub struct Synth {
    envelopes: [Envelope; NUMBER_OF_VOICES],
    oscillators: [HarmonicOscillator; NUMBER_OF_VOICES],
    midibuffer: PolyMidiBuffer,

    oscillator: Oscillator,
    oscillator2: HarmonicOscillator,
    delay: DelayLine,
    low_pass: Biquad,
    //parameters
    volume: f32,
}

impl Synth {
    pub fn default(sample_rate: f32) -> Self {
        Synth {
            oscillator: Oscillator::new(sample_rate),
            oscillator2: HarmonicOscillator::new(sample_rate, 500. ),
            envelopes: [Envelope::new(sample_rate as i32); NUMBER_OF_VOICES],
            oscillators: [HarmonicOscillator::new(sample_rate, 500.); NUMBER_OF_VOICES],
            midibuffer: PolyMidiBuffer::new(NUMBER_OF_VOICES),
            low_pass: Biquad::new(sample_rate, crate::filter::FilterType::LPF),
            delay: DelayLine::new(sample_rate, buffer::MAXIMUM_DELAY_TIME),
            volume: 0.5,
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
                None => self.envelopes[i].note_off(),
                Some(midi_note) => {
                    self.envelopes[i].note_on();
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
        let mut sample: f32 = 0.;
        for i in VOICE_ITERATOR {
            match self.envelopes[i].status {
                envelope::Segment::Off => {}
                _ => sample += self.oscillators[i].process() * self.envelopes[i].process(),
            }
        }
        // sample = self.oscillators[1].process();
        // sample = self.oscillator2.process();
        // sample = self.oscillator.process();
        sample /= 4.;
        sample = self.low_pass.process(sample);
        // EFFECTS
        sample = self.delay.process(sample);
        
        //vca
        sample *= self.volume;

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

    
    pub fn set_parameter(&mut self, (id, new_value): ParameterUpdate) {

        //need to find the parameter description to know the min max

        type ID = ParameterID;
        match id {
            ID::Volume => self.volume = new_value,
            //oscillator
            ID::OscHarmonicGain => //self.oscillator.gain_exponent=new_value, 
            self
                .oscillators
                .iter_mut()
                .for_each(|osc| osc.harmonic_gain_exponent=new_value),
            ID::OscHarmonicRatio => //self.oscillator.harmonic_index_increment=new_value, 
            self
                .oscillators
                .iter_mut()
                .for_each(|osc| osc.harmonic_index_increment=new_value),
            // envelope
            ID::EnvelopeAttack => self
                .envelopes
                .iter_mut()
                .for_each(|env| env.set_attack(new_value)),
            ID::EnvelopeRelease => self
                .envelopes
                .iter_mut()
                .for_each(|env| env.set_release(new_value)),
            ID::FilterCutoff => self.low_pass.set_frequency(new_value),
            //delay
            ID::DelayDryWet=>self.delay.set_dry_wet(new_value),
            ID::DelayTime=>self.delay.set_time(new_value),
            ID::DelayFeedback=>{
                self.delay.set_freeze(new_value > 0.95);
                self.delay.set_feedback(new_value)
            }

        }
    }
}
