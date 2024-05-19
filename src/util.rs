use std::iter::Peekable;

use crate::scanner::Token;

#[derive(Clone)]
pub struct PrevPeekable<I>
where
    I: Iterator,
    <I as Iterator>::Item: Clone,
{
    inner: Peekable<I>,
    previous: Option<I::Item>,
}

impl<I> PrevPeekable<I>
where
    I: Iterator,
    <I as Iterator>::Item: Clone,
{
    pub fn from(inner: I) -> PrevPeekable<I> {
        PrevPeekable {
            inner: inner.peekable(),
            previous: None,
        }
    }

    pub fn prev_unwrap(&self) -> I::Item {
        self.previous.clone().unwrap()
    }

    pub fn peek(&mut self) -> Option<&I::Item> {
        self.inner.peek()
    }
}

impl<I> Iterator for PrevPeekable<I>
where
    I: Iterator,
    <I as Iterator>::Item: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.next();
        self.previous = next.clone();
        next
    }
}
