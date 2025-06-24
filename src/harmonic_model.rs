use crate::buffer;
use crate::buffer::DelayLine;
use crate::envelope;
use crate::envelope::Envelope;
use crate::midi::MidiMessage;
use crate::midibuffer::PolyMidiBuffer;
use crate::parameters::ParameterCapsule;
use crate::reverb::Reverb;
use crate::synth::HasEngine;
use crate::synth::HasMidiInput;
use crate::synth::HasParameters;
use crate::Biquad;
use crate::HarmonicOscillator;
// type ID = ParameterID;
use crate::num;
use crate::num_derive;
use crate::parameters::Parameters;
use crate::ParameterUpdate;
use num_derive::FromPrimitive;

const NUMBER_OF_VOICES: usize = 4;
const VOICE_ITERATOR: std::ops::Range<usize> = 0..NUMBER_OF_VOICES;

const NB_SYNTH_PARAM: usize = 11;

#[derive(PartialEq, Debug, Copy, Clone, FromPrimitive)] //from primitive allow me to cast i32 as enum
pub enum HarmonicModelParamID {
    OscHarmonicRatio,
    OscHarmonicGain,
    EnvelopeAttack,
    EnvelopeRelease,
    FilterCutoff,
    DelayTime,
    DelayFeedback,
    DelayDryWet,
    ReverbTime,
    ReverbDryWet,
    Volume,
}

pub struct HarmonicModel {
    envelopes: [Envelope; NUMBER_OF_VOICES],
    oscillators: [HarmonicOscillator; NUMBER_OF_VOICES],
    midibuffer: PolyMidiBuffer,
    reverb: Reverb,
    delay: DelayLine,
    low_pass: Biquad,
    //parameters
    volume: f32,
}

impl HarmonicModel {
    pub fn new(sample_rate: f32) -> Self {
        HarmonicModel {
            reverb: Reverb::new(sample_rate),
            envelopes: [Envelope::new(sample_rate as i32); NUMBER_OF_VOICES],
            oscillators: [HarmonicOscillator::new(sample_rate, 500.); NUMBER_OF_VOICES],
            midibuffer: PolyMidiBuffer::new(NUMBER_OF_VOICES),
            low_pass: Biquad::new(sample_rate, crate::filter::FilterType::LPF),
            delay: DelayLine::new(sample_rate, buffer::MAXIMUM_DELAY_TIME),
            volume: 0.5,
            // parameters: Parameters {},
        }
    }
}

impl HasMidiInput for HarmonicModel {
    fn set_note(&mut self, message: MidiMessage) {
        match message {
            MidiMessage::NoteOff(midi_note) => self.midibuffer.remove_note(midi_note),
            MidiMessage::NoteOn(midi_note) => self.midibuffer.add_note(midi_note),
            _ => {}
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
}

impl HasEngine for HarmonicModel {
    fn process(&mut self) -> f32 {
        let mut sample: f32 = 0.;
        for i in VOICE_ITERATOR {
            match self.envelopes[i].status {
                envelope::Segment::Off => {}
                _ => sample += self.oscillators[i].process() * self.envelopes[i].process(),
            }
        }
        sample /= 4.;
        sample = self.low_pass.process(sample);
        // EFFECTS
        sample = self.delay.process(sample);
        sample = self.reverb.process(sample);

        //vca
        sample *= self.volume;

        return sample;
    }
}

impl HasParameters for HarmonicModel {
    fn get_parameters() -> Parameters {
        type ID = HarmonicModelParamID;
        type P = ParameterCapsule;

        let params = Parameters {
            parameters: vec![
                P::new(
                    ID::OscHarmonicRatio as i32,
                    "osc-hrmrat",
                    32,
                    'h',
                    0.2,
                    2.,
                    1.4,
                ),
                P::new(
                    ID::OscHarmonicGain as i32,
                    "osc-hrmgn",
                    32,
                    'g',
                    0.01,
                    3.,
                    1.,
                ),
                //envelope
                P::new(
                    ID::EnvelopeAttack as i32,
                    "env-atk",
                    3,
                    'a',
                    envelope::MINIMUM_ENVELOPE_TIME,
                    envelope::MAXIMUM_ENVELOPE_TIME,
                    2.,
                ),
                P::new(
                    ID::EnvelopeRelease as i32,
                    "env-dcy",
                    3,
                    'd',
                    envelope::MINIMUM_ENVELOPE_TIME,
                    envelope::MAXIMUM_ENVELOPE_TIME,
                    2.,
                ),
                P::new(ID::FilterCutoff as i32, "cutoff", 36, 'c', 20., 20000., 4.),
                //Delay
                P::new(
                    ID::DelayTime as i32,
                    "dly-time",
                    4,
                    't',
                    buffer::MINIMUM_DELAY_TIME,
                    buffer::MAXIMUM_DELAY_TIME,
                    2.,
                ),
                P::new(ID::DelayFeedback as i32, "dly-feed", 4, 'f', 0., 1.0, 1.),
                P::new(ID::DelayDryWet as i32, "dly-wet", 0, 'w', 0., 1., 1.),
                P::new(ID::ReverbDryWet as i32, "rvb-wet", 0, 'r', 0., 1., 1.),
                P::new(ID::ReverbTime as i32, "rvb-time", 0, '9', 0., 0.99, 1.),
                //global
                P::new(ID::Volume as i32, "volume", 14, 'v', 0., 2., 2.),
            ],
            nb_param: NB_SYNTH_PARAM,
        };
        assert!(params.no_id_double());
        assert!(params.no_cc_double());
        params
    }

    fn set_parameter(&mut self, (id, new_value): ParameterUpdate) {
        //need to find the parameter description to know the min max

        type ID = HarmonicModelParamID;
        let typed_id: HarmonicModelParamID = num::FromPrimitive::from_i32(id).unwrap();
        match typed_id {
            ID::Volume => self.volume = new_value,
            ID::ReverbDryWet => self.reverb.set_reverb_time(new_value),
            ID::ReverbTime => self.reverb.dry_wet = new_value,
            //oscillator
            ID::OscHarmonicGain => self
                .oscillators
                .iter_mut()
                .for_each(|osc| osc.harmonic_gain_exponent = new_value),
            ID::OscHarmonicRatio => self
                .oscillators
                .iter_mut()
                .for_each(|osc| osc.harmonic_index_increment = new_value),
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
            ID::DelayDryWet => self.delay.set_dry_wet(new_value),
            ID::DelayTime => self.delay.set_delay_time(new_value),
            ID::DelayFeedback => {
                self.delay.set_freeze(new_value > 0.99);
                self.delay.set_feedback(new_value)
            }
        }
    }
}

// trait Synth: HasParameters + HasEngine + HasMidiInput;
