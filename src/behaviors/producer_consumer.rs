use tokio::sync::mpsc::channel;
use crate::Status;
use tokio::sync::mpsc::Sender;
use async_trait::async_trait;

use super::{Dispatcher, DestinationDown, DispatcherType};



#[async_trait]
pub trait ProducerConsumer<In, Out> {
    
    /// init used for initialize producer
    async fn init(&mut self);

    /// receive events from upstream and return events as downstream to next destination 
    async fn handle_events(&mut self, upstream_events: Vec<In>) -> Vec<Out>;

    async fn terminate(&mut self);
}


// -----------------------------------------

pub struct ProducerConsumerRunnable<In, Out> {
    proc         : Box<dyn ProducerConsumer<In, Out> + Send>,
    dispatcher   : Dispatcher<Out>

}


impl<In, Out> ProducerConsumerRunnable<In, Out> 
where
    In:  Clone + Send + 'static,
    Out: Clone + Send + 'static 
{
    pub fn new(proc            : Box<dyn ProducerConsumer<In, Out> + Send>,
               subscribe_to    : Vec<Sender<Vec<Out>>>,
               dispatcher_type : Option<DispatcherType>) 
               
    ->  Result<Self, Status>
    
    {

        // Check subscribe_to not be empty
        if subscribe_to.len() == 0 {
            return Err(Status::SenderNotFound);
        }

        // if dispatcher_type is None, set RoundRobin
        let dt = if let Some(dt) = dispatcher_type { dt } 
                                else { DispatcherType::RoundRobin };
        
        
        // Check subscribe_to not have duplicate sender
        match Dispatcher::new(subscribe_to, dt) {
            Ok(dispatcher) => {

                Ok(Self {
                    proc,
                    dispatcher
                })        
            }
            Err(a) => {
                Err(a)
            }
        }
        
    }



    /// produce events and send to dst/subscribe_to by dispatcher
    #[inline]
    pub async fn produce_to_dst(&mut self, upstream_events: Vec<In>) -> Result<(), DestinationDown<Out>> {
        let events = self.proc.handle_events(upstream_events).await;
        self.dispatcher.dispatch(events).await
    }

    


    #[inline]
    pub fn run(mut self, buffer: usize) -> Sender<Vec<In>> {
        let (sx, mut rx) = channel(buffer);

        tokio::spawn(async move {
            
            self.proc.init().await;

            loop {

                // Listen on channel
                match rx.recv().await {
                    Some(upstream_events) => {

                        // produce events and dispatch
                        match self.produce_to_dst(upstream_events).await {
                            Err(dd) => return Some(dd),
                            Ok(_) => (),
                        }              
                        
                    }
                    None => {
                        // upstream terminate
                        self.proc.terminate().await;
                        return None
                    }
                }
  
            }
        });

        sx
    }
}

