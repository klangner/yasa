//! A simple FM receiver that you can tune to nearby radio stations
//!
//! When you run the example, it will build a flowgraph consisting of the following blocks:
//! * SeifySource: Gets data from your SDR
//! * Demodulator: Demodulates the FM signal
//! * AudioSink: Plays the demodulated signal on your device
//!
//! After giving it some time to start up the SDR, it enters a loop where you will
//! be periodically asked to enter a new frequency that the SDR will be tuned to.
//! **Watch out** though: Some frequencies (very high or very low) might be unsupported
//! by your SDR and may cause a crash.


use futuresdr::anyhow::Result;
use futuresdr::async_io;
use futuresdr::blocks::audio::AudioSink;
use futuresdr::blocks::seify::SourceBuilder;
use futuresdr::blocks::Apply;
use futuresdr::blocks::FirBuilder;
use futuresdr::futuredsp::firdes;
use futuresdr::log;
use futuresdr::macros::connect;
use futuresdr::num_complex::Complex32;
use futuresdr::num_integer::gcd;
use futuresdr::runtime::Block;
use futuresdr::runtime::Pmt;
use futuresdr::runtime::{Flowgraph, FlowgraphHandle, Runtime};


pub struct FMRadio {
    handle: FlowgraphHandle,
    source: SourceBlock,
}

enum SourceBlock {
    Seify { id: usize, freq_offset: f64, freq_port_id: usize },
}

impl FMRadio {
    pub fn start(frequency: f64, gain: f64, rate: f64, args: &str) -> Result<Self> {
        let freq_offset = rate / 4.0;

        let mut audio_rates = AudioSink::supported_sample_rates();
        assert!(!audio_rates.is_empty());
        audio_rates.sort_by_key(|a| std::cmp::Reverse(gcd(*a, rate as u32)));
        let audio_rate = audio_rates[0];
        log::info!("Selected Audio Rate {audio_rate:?} from supported {audio_rates:?}");

        // Create a new Seify SDR block with the given parameters
        let src = FMRadio::seify(frequency + freq_offset, gain, rate, args).expect("Can't init Seify");

        // Downsample before demodulation
        // why do we need this?
        let mut audio_mult = 5;
        while (audio_mult * audio_rate) as f64 > freq_offset + 100e3 {
            audio_mult -= 1;
        }
        log::info!("Audio Mult {audio_mult:?}");

        let shift = FMRadio::shift(freq_offset, rate);

        let interp = (audio_rate * audio_mult) as usize;
        let decim = rate as usize;
        log::info!("interp {interp}   decim {decim}");
        let resamp1 = FirBuilder::new_resampling::<Complex32, Complex32>(interp, decim);

        let demod = FMRadio::fm_demodulation();

        let audio_filter = FMRadio::audio_filter(audio_rate, audio_mult);

        // Single-channel `AudioSink` with the downsampled rate (sample_rate / (8*5) = 48_000)
        let snk = AudioSink::new(audio_rate, 1);
                
        // Save ports for connectiong to the blocks
        let freq_port_id = src.message_input_name_to_id("freq").unwrap();

        // Create the `Flowgraph` and add `Block`s
        let runtime = Runtime::new();
        let mut fg = Flowgraph::new();
        connect!(fg, src > shift > resamp1 > demod > audio_filter > snk.in;);

        // Start the flowgraph and save the handle
        let (_res, handle) = runtime.start_sync(fg);
        let source_block = SourceBlock::Seify { id: src, freq_offset, freq_port_id };
        
        Ok(Self { 
            handle, 
            source: source_block,
        })

    }

    pub fn tune_to(&mut self, new_freq: f64) -> Result<()> {
        match self.source {
            SourceBlock::Seify { id, freq_offset, freq_port_id } => {
                log::info!("Tune to: {}", new_freq);
                async_io::block_on(self.handle.call(
                    id,
                    freq_port_id,
                    Pmt::F64(new_freq + freq_offset),
                ))?;
            }
        }
        Ok(())
    }

    // Build Seify block.
    fn seify(frequency: f64, gain: f64, rate: f64, args: &str) -> Result<Block> {
        SourceBuilder::new()
            .args(args)?
            .frequency(frequency)
            .sample_rate(rate)
            .gain(gain)
            .build()
    } 

    // Shift signal by a given offset
    fn shift(freq_offset: f64, rate: f64) -> Block {
        let mut last = Complex32::new(1.0, 0.0);
        let add = Complex32::from_polar(
            1.0,
            (2.0 * std::f64::consts::PI * freq_offset / rate) as f32,
        );
        let shift = Apply::new(move |v: &Complex32| -> Complex32 {
            last *= add;
            last * v
        });
        
        shift
    }

    // Demodulation block using the conjugate delay method
    // See https://en.wikipedia.org/wiki/Detector_(radio)#Quadrature_detector
    fn fm_demodulation() -> Block {
        let mut last = Complex32::new(0.0, 0.0); // store sample x[n-1]
        let demod = Apply::new(move |v: &Complex32| -> f32 {
            let arg = (v * last.conj()).arg(); // Obtain phase of x[n] * conj(x[n-1])
            last = *v;
            arg
        });

        demod
    }

    // Design filter for the audio and decimate by 5.
    // Ideally, this should be a FM de-emphasis filter, but the following works.
    fn audio_filter(audio_rate: u32, audio_mult: u32) -> Block {
        let cutoff = 2_000.0 / (audio_rate * audio_mult) as f64;
        let transition = 10_000.0 / (audio_rate * audio_mult) as f64;
        log::info!("cutoff {cutoff}   transition {transition}");
        let audio_filter_taps = firdes::kaiser::lowpass::<f32>(cutoff, transition, 0.1);
        let resamp = FirBuilder::new_resampling_with_taps::<f32, f32, _, _>(
            1,
            audio_mult as usize,
            audio_filter_taps,
        );

        resamp
    }
}