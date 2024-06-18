/*
  ==============================================================================

    Outils.h
    Created: 11 Mar 2023 5:41:09pm
    Author:  thoma

  ==============================================================================
*/

/// @brief Slow value change of a parameter, slew factor working best between
/// 0.8 - 0.99
/// @param new_value
/// @param old_value
/// @param slew_factor a bigger slew factor means a slower change, must be <1 to
/// keep stability
/// @return
/// 


static ORCA_CHARACTERS: [char; 36] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z'];

pub fn slew_value(new_value: f32, old_value: f32, slew_factor: f32) -> f32 {
    return (new_value * (1.0 - slew_factor)) + (old_value * (slew_factor));
}

pub fn midi_to_frequence(midi_note: u8)->f32{
    return 440. * f32::powf(2., (midi_note as f32-69.)/12.);
}

/// @brief convert milliseconds to samples
/// @param time in seconds
/// @param sample_rate sample / secondes in Hz
/// @return
pub fn convert_ms_to_sample(time: f32, sample_rate: f32) -> f32 {
    return (sample_rate / 1000.) * time;
}

///value in seconds
pub fn rt60_to_gain(rt60: f32, loop_time: f32)->f32{
    //get gain in decibel
    let gain = -60. * loop_time / rt60;
    //convert from logarithmic gain to linear amplification
    return 10.0_f32.powf(gain / 20.0);
}

pub fn map_value_float_to_int(
    in_min: f32,
    in_max: f32,
    value: f32,
    out_min: i32,
    out_max: i32,
) -> i32 {
    let ratio = (out_max - out_min) as f32 / (in_max - in_min);
    let offset = out_min as f32 - (in_min * ratio);
    let output = (value * ratio + offset) as i32;
    return output;
}

pub fn map_value(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    let ratio = (out_max - out_min) / (in_max - in_min);
    let offset = out_min - (in_min * ratio);
    let output = value * ratio + offset;
    return output;
}

pub fn linear_crossfade(dry: f32, wet: f32, parameter: f32) -> f32 {
    return (dry * (1.0 - parameter)) + (wet * parameter);
}
/// 0 -> dry
/// 1 -> wet
pub fn equal_power_crossfade(dry: f32, wet: f32, mut parameter: f32) -> f32 {
    parameter = 1. - parameter;
    parameter = (parameter - 0.5) * 2.;
    let volumes_dry = (0.5 * (1. + parameter)).sqrt();
    let volumes_wet = (0.5 * (1. - parameter)).sqrt();
    return (dry * volumes_dry) + (wet * volumes_wet);
}

pub fn get_orca_character(value: i32)->Option<char>{
    return ORCA_CHARACTERS.get(value as usize).copied();
}

pub fn get_orca_integer(character: char)->Option<u8>{
    for i in 0..35 {
        if character == ORCA_CHARACTERS[i]{return Some(i as u8)}
    }
    None
}

#[cfg(test)]
mod test {
    use crate::outils::ORCA_CHARACTERS;

    #[test]
    fn basics() {
        let value = 0;
        print!("{}", ORCA_CHARACTERS[value as usize]);
        // Check empty list behaves right
    }
}