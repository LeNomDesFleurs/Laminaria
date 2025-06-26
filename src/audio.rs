use crate::{ midi::MidiMessage, synth::{self, HasConstructor, HasEngine, HasMidiInput, HasParameters, Synth}, HarmonicModel, ParameterUpdate};
use cpal::{
    traits::{DeviceTrait, HostTrait},
    SizedSample,
};
use cpal::{FromSample, Sample};
use std::{sync::mpsc::Receiver};

pub fn stream_setup_for(
    parameter_receiver: Receiver<ParameterUpdate>,
    midi_receiver: Receiver<MidiMessage>,
    synth_model: Box<dyn Synth>,
) -> Result<cpal::Stream, anyhow::Error>
where
{
    let (_host, device, config) = host_device_setup()?;

    let result = match config.sample_format() {
    f @ cpal::SampleFormat::I8  => make_stream::<i8>,
    f @ cpal::SampleFormat::I16 => make_stream::<i16>,
    f @ cpal::SampleFormat::I32 => make_stream::<i32>,
    f @ cpal::SampleFormat::I64 => make_stream::<i64>,
    f @ cpal::SampleFormat::U8  => make_stream::<u8>,
    f @ cpal::SampleFormat::U16 => make_stream::<u16>,
    f @ cpal::SampleFormat::U32 => make_stream::<u32>,
    f @ cpal::SampleFormat::U64 => make_stream::<u64>,
    f @ cpal::SampleFormat::F32 => make_stream::<f32>,
    f @ cpal::SampleFormat::F64 => make_stream::<f64>,
    other => return Err(anyhow::anyhow!("Unsupported sample format '{other:?}'")),
};

result(&device, &config.into(), parameter_receiver, midi_receiver, synth_model)
}

pub fn host_device_setup(
) -> Result<(cpal::Host, cpal::Device, cpal::SupportedStreamConfig), anyhow::Error> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))?;
    println!("Output device : {}", device.name()?);

    let config = device.default_output_config()?;
    println!("Default output config : {:?}", config);

    Ok((host, device, config))
}

pub fn make_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    interface_receiver: Receiver<ParameterUpdate>,
    midi_receiver: Receiver<MidiMessage>,
    mut synth_model: Box<dyn Synth>,
) -> Result<cpal::Stream, anyhow::Error>
where
    T: SizedSample + FromSample<f32>,
{
    let num_channels = config.channels as usize;

    let mut synth = synth_model.init(config.sample_rate.0 as f32);
    let err_fn = |err| eprintln!("Error building output sound stream: {}", err);

    let time_at_start = std::time::Instant::now();
    println!("Time at start: {:?}", time_at_start);
    //create the audio stream
    let stream = device.build_output_stream(
        config,
        //check for new parameter values
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            if let Ok((id, value)) = interface_receiver.try_recv() {
                synth_model.set_parameter((id, value))
            }
            //check for new midi value
            if let Ok(message) = midi_receiver.try_recv() {
                synth_model.set_note(message);
            }
            //process buffer
            process_frame(output, &mut synth_model, num_channels)
        },
        err_fn,
        None,
    )?;

    Ok(stream)
}

fn process_frame<SampleType>(output: &mut [SampleType], synth_model: &mut Box<dyn Synth>, num_channels: usize)
where
    SampleType: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(num_channels) {
        let value: SampleType = SampleType::from_sample(synth_model.process());

        // copy the same value to all channels
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
