use super::{Pipe, Producer};

#[derive(Debug)]
pub struct ChainedPipe<I, O, Shared, Error> {
    input: I,
    output: O,
    _s: std::marker::PhantomData<(Shared, Error)>,
}

impl<I, O, Shared, Error> ChainedPipe<I, O, Shared, Error> {
    pub fn new(input: I, output: O) -> Self {
        Self {
            input,
            output,
            _s: std::marker::PhantomData,
        }
    }
}

impl<I, O> Producer for ChainedPipe<I, O, I::Item, O::Error>
where
    I: Producer,
    O: for<'a> Pipe<Input = I::Item>,
{
    type Item = Result<O::Output, O::Error>;

    fn produce(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.input.produce() {
            match Pipe::process(&mut self.output, item) {
                output @ Ok(Some(_)) => return output.transpose(),
                err @ Err(_) => return err.transpose(),
                _ => {}
            }
        }
        self.output.close();
        None
    }
}

impl<I, O, Shared, Error> Pipe for ChainedPipe<I, O, Shared, Error>
where
    I: Pipe<Output = Shared, Error = Error>,
    O: for<'a> Pipe<Input = Shared, Error = Error>,
{
    type Input = I::Input;
    type Output = O::Output;
    type Error = O::Error;

    fn process(&mut self, input: Self::Input) -> Result<Option<Self::Output>, Self::Error> {
        self.input.process(input).and_then(|input| {
            input
                .map(|input| self.output.process(input))
                .transpose()
                .map(Option::flatten)
        })
    }

    fn close(&mut self) {
        self.input.close();
        self.output.close();
    }
}
