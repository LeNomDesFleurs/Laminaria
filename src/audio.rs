
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SizedSample,
};
use cpal::{FromSample, Sample};
use std::sync::mpsc::{Receiver, SyncSender};
use crate::parameters::Parameter;
use crate::Synth;
use std::collections::HashMap;

pub fn stream_setup_for(parameter_receiver: Receiver<Parameter>, midi_receiver: Receiver<[u8;3]>) -> Result<cpal::Stream, anyhow::Error>
where
{
    let (_host, device, config) = host_device_setup()?;

    match config.sample_format() {
        cpal::SampleFormat::I8 => make_stream::<i8>(&device, &config.into(), parameter_receiver, midi_receiver),
        cpal::SampleFormat::I16 => make_stream::<i16>(&device, &config.into(), parameter_receiver, midi_receiver),
        cpal::SampleFormat::I32 => make_stream::<i32>(&device, &config.into(), parameter_receiver, midi_receiver),
        cpal::SampleFormat::I64 => make_stream::<i64>(&device, &config.into(), parameter_receiver, midi_receiver),
        cpal::SampleFormat::U8 => make_stream::<u8>(&device, &config.into(), parameter_receiver, midi_receiver),
        cpal::SampleFormat::U16 => make_stream::<u16>(&device, &config.into(), parameter_receiver, midi_receiver),
        cpal::SampleFormat::U32 => make_stream::<u32>(&device, &config.into(), parameter_receiver, midi_receiver),
        cpal::SampleFormat::U64 => make_stream::<u64>(&device, &config.into(), parameter_receiver, midi_receiver),
        cpal::SampleFormat::F32 => make_stream::<f32>(&device, &config.into(), parameter_receiver, midi_receiver),
        cpal::SampleFormat::F64 => make_stream::<f64>(&device, &config.into(), parameter_receiver, midi_receiver),
        sample_format => Err(anyhow::Error::msg(format!(
            "Unsupported sample format '{sample_format}'"
        ))),
    }
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
    interface_receiver: Receiver<Parameter>,
    midi_receiver: Receiver<[u8;3]>,
) -> Result<cpal::Stream, anyhow::Error>
where
    T: SizedSample + FromSample<f32>,
{
    let num_channels = config.channels as usize;

    let mut synth = Synth::default(config.sample_rate.0 as f32);
    let err_fn = |err| eprintln!("Error building output sound stream: {}", err);

    let time_at_start = std::time::Instant::now();
    println!("Time at start: {:?}", time_at_start);
//create the audio stream
    let stream = device.build_output_stream(
        config,
//check for new parameter values
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            if let Ok(parameter) = interface_receiver.try_recv() {
                let name = &parameter.name;
                if let Some(x) = synth.parameters.get_mut(&name as &str) {
                    *x = parameter;
                }
            }
//check for new midi value
            if let Ok(midi) = midi_receiver.try_recv(){
                if midi[0]==144{
                    synth.set_note(midi[1], true);
                }
                else if midi[0]==128 {
                    synth.set_note(midi[1], false);
                }
            }
//process buffer
            process_frame(output, &mut synth, num_channels)
        },
        err_fn,
        None,
    )?;

    Ok(stream)
}

fn process_frame<SampleType>(output: &mut [SampleType], synth: &mut Synth, num_channels: usize)
where
    SampleType: Sample + FromSample<f32>,
{

    for frame in output.chunks_mut(num_channels) {
        // let value: SampleType = SampleType::from_sample(oscillator.tick());
        let value: SampleType = SampleType::from_sample(synth.tick());
        // let value: SampleType = SampleType::from_sample(lfo.tick());
        // copy the same value to all channels
        for sample in frame.iter_mut() {
            // oscillator.
            *sample = value;
        }
        // println!("{}", iterator)
    }
}

fn die(e: &std::io::Error) {
    panic!("{}", e);
}

