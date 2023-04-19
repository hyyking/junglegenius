use super::Pipe;
use crate::Error;

#[derive(Debug)]
pub struct CloneSplit<T>(std::marker::PhantomData<T>);

impl<T> CloneSplit<T> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<T> Pipe for CloneSplit<T>
where
    T: Clone,
{
    type Input = T;

    type Output = (T, T);

    type Error = Error;

    fn process(&mut self, input: Self::Input) -> Result<Option<Self::Output>, Self::Error> {
        Ok(Some((input.clone(), input)))
    }
    fn close(&mut self) {}
}
#[derive(Debug)]
pub struct ConsumeLeft<C, T> {
    consummer: C,
    _s: std::marker::PhantomData<T>,
}

impl<C, T> ConsumeLeft<C, T> {
    pub fn new(consummer: C) -> Self {
        Self {
            consummer,
            _s: std::marker::PhantomData,
        }
    }
}

impl<C, T> Pipe for ConsumeLeft<C, T>
where
    C: Pipe<Output = ()>,
{
    type Input = (C::Input, T);

    type Output = T;

    type Error = C::Error;

    fn process(&mut self, input: Self::Input) -> Result<Option<Self::Output>, Self::Error> {
        self.consummer.process(input.0)?;
        Ok(Some(input.1))
    }

    fn close(&mut self) {
        self.consummer.close()
    }
}
