mod chained;
mod split;
pub use chained::ChainedPipe;
pub use split::{CloneSplit, ConsumeLeft};

use crate::Error;

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

#[derive(Debug)]
pub struct TryCollector<P: Producer, C> {
    _s: std::marker::PhantomData<(P, C)>,
}

impl<P: Producer, C> TryCollector<P, C> {
    pub fn new() -> Self {
        Self {
            _s: std::marker::PhantomData,
        }
    }
}

impl<O, P, C> Pipe for TryCollector<P, C>
where
    P: Producer<Item = Result<O, Error>>,
    C: FromIterator<O>,
{
    type Input = P;

    type Output = C;

    type Error = Error;

    fn process(&mut self, mut input: Self::Input) -> Result<Option<Self::Output>, Self::Error> {
        Result::<C, Self::Error>::from_iter(std::iter::from_fn(|| input.produce())).map(Some)
    }
}

#[derive(Debug)]
pub struct OwnedProducer<P: Producer>(Option<P>);

impl<P: Producer> Pipe for OwnedProducer<P> {
    type Input = ();

    type Output = P;

    type Error = Error;

    fn process(&mut self, _: Self::Input) -> Result<Option<Self::Output>, Self::Error> {
        Ok(self.0.take())
    }
}

impl<T: Producer> Producer for OwnedProducer<T> {
    type Item = T;

    fn produce(&mut self) -> Option<Self::Item> {
        self.0.take()
    }
}

pub trait Producer {
    type Item;
    fn produce(&mut self) -> Option<Self::Item>;

    fn producer(self) -> OwnedProducer<Self>
    where
        Self: Sized,
    {
        OwnedProducer(Some(self))
    }

    fn feed<P>(self, other: P) -> ChainedPipe<Self, P, Self::Item, P::Error>
    where
        Self: Sized,
        P: for<'a> Pipe<Input = Self::Item>,
    {
        ChainedPipe::new(self, other)
    }

    fn chain<P>(self, other: P) -> ChainedProducer<Self, P>
    where
        Self: Sized,
        P: Producer<Item = Self::Item>,
    {
        ChainedProducer { a: self, b: other }
    }
}

pub struct ChainedProducer<A, B> {
    a: A,
    b: B,
}

impl<A, B> Producer for ChainedProducer<A, B>
where
    A: Producer,
    B: Producer<Item = A::Item>,
{
    type Item = A::Item;

    fn produce(&mut self) -> Option<Self::Item> {
        self.a.produce().or_else(|| self.b.produce())
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
