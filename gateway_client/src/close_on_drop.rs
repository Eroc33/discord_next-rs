use std::{
    marker::PhantomData,
    pin::Pin,
    task::*,
};
use futures::{Sink,SinkExt};

pub (crate) struct CloseOnDrop<S: Sink<I> + Unpin,I>(S,PhantomData<I>);

impl<S,I> CloseOnDrop<S,I>
    where S: Sink<I> + Unpin,
{
    pub fn new(s: S) -> Self{
        Self(s,PhantomData)
    }
}

impl<S,I> Sink<I> for CloseOnDrop<S,I>
    where S: Sink<I> + Unpin
{
    type Error = S::Error;
    fn start_send(self: Pin<&mut Self>, item: I) -> Result<(), Self::Error>{
        Sink::start_send(unsafe{self.map_unchecked_mut(|s| &mut s.0)}, item)
    }
    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>>{
        Sink::poll_ready(unsafe{self.map_unchecked_mut(|s| &mut s.0)}, cx)
    }
    fn poll_flush(
        self: Pin<&mut Self>, 
        cx: &mut Context
    ) -> Poll<Result<(), Self::Error>>{
        Sink::poll_flush(unsafe{self.map_unchecked_mut(|s| &mut s.0)}, cx)
    }
    fn poll_close(
        self: Pin<&mut Self>, 
        cx: &mut Context
    ) -> Poll<Result<(), Self::Error>>{
        Sink::poll_close(unsafe{self.map_unchecked_mut(|s| &mut s.0)}, cx)
    }
}

impl<I,S> Drop for CloseOnDrop<S,I>
    where S: Sink<I> + Unpin
{
    fn drop(&mut self){
        //we ignore this since we can't do anything about it if it fails, and we're only sending the close signal to be courteous
        let _ = self.0.close();
    }
}