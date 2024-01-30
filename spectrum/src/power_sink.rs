use futuresdr::anyhow::Result;
use futuresdr::macros::async_trait; 
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::{Block, BlockMeta, BlockMetaBuilder, Kernel, MessageIo, MessageIoBuilder, StreamIo, StreamIoBuilder, WorkIo};


/// Store received samples in vector.
pub struct PowerSink<> {
}

impl PowerSink {
    /// Create VectorSink block
    pub fn new() -> Block {
        Block::new(
            BlockMetaBuilder::new("PowerSink").build(),
            StreamIoBuilder::new().add_input::<Complex32>("in").build(),
            MessageIoBuilder::<Self>::new().build(),
            PowerSink {
            },
        )
    }
}

#[doc(hidden)]
#[async_trait]
impl Kernel for PowerSink {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let xs = sio.input(0).slice::<Complex32>();

        println!("len: {}", xs.len());
        let power: f32 = if xs.len() > 0 {
            // 20.0 * (xs[0].norm() / i8::MAX as f32).log10()
            xs.iter().map(|c| (20.0 * (c.norm() / 127.0).log10()) as i32).max().unwrap() as f32
        } else {
            0.0
        };

        if power > -40.0 {
            println!("Power: {}", power);
        }

        sio.input(0).consume(xs.len());

        if sio.input(0).finished() {
            io.finished = true;
        }

        Ok(())
    }
}