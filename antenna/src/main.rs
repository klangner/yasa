mod power_sink;

use clap::Parser;
use dsp::num_complex::Complex32;
use futuresdr::anyhow::Result;
use futuresdr::blocks::seify::SourceBuilder;
use futuresdr::blocks::Apply;
use futuresdr::blocks::Fft;
use futuresdr::macros::connect;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;

use crate::power_sink::PowerSink;


const SAMPLE_RATE: f64 = 2_000_000.;
const FFT_SIZE: usize = 4096;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Gain to apply to the source
    #[clap(short, long, default_value_t = 50.0)]
    gain: f64,

    /// Measure frequency in MHz
    #[clap(short, long, default_value_t = 100.0)]
    frequency: f64,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Freq: {}", args.frequency);
    let mut fg = Flowgraph::new();
    let src_block = SourceBuilder::new()
        .frequency(args.frequency * 1e6)
        .sample_rate(SAMPLE_RATE)
        .gain(args.gain)
        .build()?;
    let fft = Fft::with_options(
        FFT_SIZE,
        futuresdr::blocks::FftDirection::Forward,
        true,
        None,
    );
    let power = Apply::new(|x: &Complex32| 20.0 * (x.norm() / 127.0).log10());
    // let power = Apply::new(|x: &Complex32| x.norm_sqr());
    // let power = Apply::new(|x: &Complex32| 10.0 * x.norm_sqr().log10());
    let sink = PowerSink::new();

    connect!(fg, src_block > fft > power > sink;);

    Runtime::new().run(fg)?;

    Ok(())
}