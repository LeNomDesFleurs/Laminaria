
pub fn stream_setup_for(interface_receiver: Receiver<TextCharacteristic>, midi_receiver: Receiver<[u8;3]>) -> Result<cpal::Stream, anyhow::Error>
where
{
    let (_host, device, config) = host_device_setup()?;

    match config.sample_format() {
        cpal::SampleFormat::I8 => make_stream::<i8>(&device, &config.into(), interface_receiver, midi_receiver),
        cpal::SampleFormat::I16 => make_stream::<i16>(&device, &config.into(), interface_receiver, midi_receiver),
        cpal::SampleFormat::I32 => make_stream::<i32>(&device, &config.into(), interface_receiver, midi_receiver),
        cpal::SampleFormat::I64 => make_stream::<i64>(&device, &config.into(), interface_receiver, midi_receiver),
        cpal::SampleFormat::U8 => make_stream::<u8>(&device, &config.into(), interface_receiver, midi_receiver),
        cpal::SampleFormat::U16 => make_stream::<u16>(&device, &config.into(), interface_receiver, midi_receiver),
        cpal::SampleFormat::U32 => make_stream::<u32>(&device, &config.into(), interface_receiver, midi_receiver),
        cpal::SampleFormat::U64 => make_stream::<u64>(&device, &config.into(), interface_receiver, midi_receiver),
        cpal::SampleFormat::F32 => make_stream::<f32>(&device, &config.into(), interface_receiver, midi_receiver),
        cpal::SampleFormat::F64 => make_stream::<f64>(&device, &config.into(), interface_receiver, midi_receiver),
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
    interface_receiver: Receiver<TextCharacteristic>,
    midi_receiver: Receiver<[u8;3]>,
) -> Result<cpal::Stream, anyhow::Error>
where
    T: SizedSample + FromSample<f32>,
{
    let num_channels = config.channels as usize;
    let args: Vec<String> = env::args().collect();

    let mut synth = Synth::default(config.sample_rate.0 as f32);
    let err_fn = |err| eprintln!("Error building output sound stream: {}", err);

    let time_at_start = std::time::Instant::now();
    println!("Time at start: {:?}", time_at_start);
    let iterator = 0;
    let mut amplitude: f32 = 0.;

    let stream = device.build_output_stream(
        config,
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            if let Ok(textcarac) = interface_receiver.try_recv() {
                synth.mapping(textcarac);
            }
            if let Ok(midi) = midi_receiver.try_recv(){
                if midi[0]==144{
                    amplitude = 1.}
                    else if midi[0]==128 {
                        amplitude = 0.
                    }
            }
            process_frame(output, &mut synth, num_channels, amplitude)
        },
        err_fn,
        None,
    )?;

    Ok(stream)
}

fn process_frame<SampleType>(output: &mut [SampleType], synth: &mut Synth, num_channels: usize, amplitude: f32)
where
    SampleType: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(num_channels) {
        // let value: SampleType = SampleType::from_sample(oscillator.tick());
        let value: SampleType = SampleType::from_sample(synth.tick()*amplitude);
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

