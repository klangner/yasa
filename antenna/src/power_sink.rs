use futuresdr::anyhow::Result;
use futuresdr::macros::async_trait;
use futuresdr::runtime::BlockMeta;
use futuresdr::runtime::BlockMetaBuilder;
use futuresdr::runtime::Kernel;
use futuresdr::runtime::MessageIo;
use futuresdr::runtime::MessageIoBuilder;
use futuresdr::runtime::StreamIo;
use futuresdr::runtime::StreamIoBuilder;
use futuresdr::runtime::WorkIo;
use futuresdr::runtime::{Block, TypedBlock};

pub struct PowerSink {
    pos: usize,
    total: f64,
}

impl PowerSink {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Block {
        Block::from_typed(Self::new_typed())
    }

    pub fn new_typed() -> TypedBlock<Self> {
        TypedBlock::new(
            BlockMetaBuilder::new("CrossbeamSink").build(),
            StreamIoBuilder::new().add_input::<f32>("in").build(),
            MessageIoBuilder::<Self>::new().build(),
            PowerSink { pos: 0, total: 0. },
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
        let i = sio.input(0).slice::<f32>();

        if i.len() == 4096 {
            // let skip = (i.len() - 1024) / 2;
            let xs: Vec<f32> = i.iter()
                .skip(2000)
                .take(46)
                .filter(|&v| v.is_finite())
                .map(|&v| v)
                .collect::<Vec<f32>>();
            let avg_power = xs.iter().sum::<f32>() / i.len() as f32;
            if self.pos < 1000 {
                self.total += avg_power as f64;
                self.pos += 1;
            } else {
                println!("Power: {}", self.total / 1000.);
                self.total = 0.;
                self.pos = 0;
            }
        }

        sio.input(0).consume(i.len());

        if sio.input(0).finished() {
            io.finished = true;
        }

        Ok(())
    }
}