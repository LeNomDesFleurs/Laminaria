# *Laminaria*, a compagnon for [ORCΛ](https://github.com/hundredrabbits/Orca)

Laminaria is a polyphonic Terminal based audio synth made in Rust.

This is a learning repo, goals are :
 - Receive MIDI
 - Generate Sound
 - Use channels, mutexes and arc to manage thread concurrency 

## 🔈 Audio demo
You can do a bunch a things but I did glitchy ambient cause that is who I am (made with two units sequenced by ORCΛ, no other effects involved)

[Click here to open the sound demo](https://lndf.fr/Media/Laminaria-sounddemo.mp3)

## 🛒 Get it

Clone this repo and `cargo build --release`, then run `Laminaria` in `target/release/`.
You can also use `cargo run` for a quick launch, but it will be less efficient.

## ⌨️ Key :
- `Esc` - quit
- `⬆ | ⬇` - select param
- `← | →` - increment or decrement parameter
- `[letter | number]` - set the value of the parameter
- `>` - Increment midi channel
- `<` - decrement midi channel
- `Tab` - select midi input

## 📺 Display :

```
[Midiport name] --- channel [channelIndex]

[cc index] - [paremeter name] - [parameter value] - ||||||||||||||||----------------
```

---

```
IAC Driver Virtual Midi 1 --- channel 0

h - osc-hrmrat - w - ||||||||||||||||||||||||||||||||--- 1.79
g - osc-hrmgn  - w - ||||||||||||||||||||||||||||||||--- 2.74
a - env-atk    - 3 - |||-------------------------------- 83.40
d - env-dcy    - 3 - |||-------------------------------- 83.40
c - cutoff     - z - ||||||||||||||||||||||||||||||||||| 20000.00
t - dly-time   - 4 - ||||------------------------------- 0.14
f - dly-feed   - 4 - ||||------------------------------- 0.11
w - dly-wet    - 0 - ----------------------------------- 0.00
r - rvb-wet    - 0 - ----------------------------------- 0.00
9 - rvb-time   - 0 - ----------------------------------- 0.00
v - volume     - e - ||||||||||||||--------------------- 0.32
```

## ⚙️ Description of the Synth

The synth has four voices. `∿Oscillators` are sinewaves banks where you manage the ratio and the gain of each harmonics (so a kind of additive synthesis). 

Amplitude Envelope are basic `ASR`. The sum of oscillators goes to a classic `low-pass` filter.

It then goes trough two `FX`, `delay` and `Reverb`. If you put the delay feedback to max, it loops the captured sound. Delay time will then pitch the sound up and down (which is the coolest thing to do with this synth).
The Reverb is just 5 allpass filters in series, there are a lot of resonance due to feedback.

## 👩🏿‍💻 Hack it

I may do a proper boilerplate someday, as of now it should already be pretty hackable :
- add parameter in the `parameter` file
  - add a key in the `Parameter ID` enum
  - increment the `NUMBER_OF_PARAMETER` const
  - add its definition in `Parameter::new()`
  - get the value of the parameter in `synth::set_parameter()`
- define the sound in `synth::process()`

## 📚 Lib :
- cpal for audio
- midir for midi
- cross term for terminal I/O

##
![picture](/Laminaria.jpg)