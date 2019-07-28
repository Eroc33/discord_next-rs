use futures::{
    channel::mpsc::{Sender,UnboundedSender, channel, unbounded},
    sink::{Sink,SinkExt as _},
    stream::StreamExt,
};

use tracing::*;

pub trait SinkExt<I>: Sink<I>{
    fn channeled(self,buffer: usize) -> Sender<I>;
    fn unbounded_channeled(self) -> UnboundedSender<I>;
}

impl<S,I,E> SinkExt<I> for S
    where S: Sink<I,Error=E> + Send + Unpin + 'static,
          I: Send + 'static,
          E: std::fmt::Debug + Send + 'static
{
    fn channeled(mut self,buffer: usize) -> Sender<I>{
        let (tx,mut rx) = channel(buffer);
        tokio::spawn(async move{
            while let Some(item) = rx.next().await{
                match self.send(item).await{
                    Ok(_) => {},
                    Err(e) => {
                        warn!("Error in channel forwarder: {:?}",e);
                    }
                }
            }
        });
        tx
    }
    fn unbounded_channeled(mut self) -> UnboundedSender<I>{
        let (tx,mut rx) = unbounded();
        tokio::spawn(async move{
            while let Some(item) = rx.next().await{
                match self.send(item).await{
                    Ok(_) => {},
                    Err(e) => {
                        warn!("Error in channel forwarder: {:?}",e);
                    }
                }
            }
        });
        tx
    }
}