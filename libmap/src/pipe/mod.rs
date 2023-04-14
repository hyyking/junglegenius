mod chained;
mod split;
pub use chained::ChainedPipe;
pub use split::{CloneSplit, ConsumeLeft};

pub trait Pipe {
    type Input;
    type Output;

    type Error;

    fn process(&mut self, input: Self::Input) -> Result<Option<Self::Output>, Self::Error>;

    fn pipe<P>(self, other: P) -> ChainedPipe<Self, P, Self::Output, Self::Error>
    where
        Self: Sized,
        P: for<'a> Pipe<Input = Self::Output, Error = Self::Error>,
    {
        ChainedPipe::new(self, other)
    }

    fn close(&mut self) {}
}

pub trait Producer {
    type Item;
    fn produce(&mut self) -> Option<Self::Item>;

    fn feed<P>(self, other: P) -> ChainedPipe<Self, P, Self::Item, P::Error>
    where
        Self: Sized,
        P: for<'a> Pipe<Input = Self::Item>,
    {
        ChainedPipe::new(self, other)
    }
}

impl<T> Producer for T
where
    T: Iterator,
{
    type Item = <T as Iterator>::Item;

    fn produce(&mut self) -> Option<Self::Item> {
        <Self as Iterator>::next(self)
    }
}
