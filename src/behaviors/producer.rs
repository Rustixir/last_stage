
use tokio::task::JoinHandle;
use tokio::sync::mpsc::Sender;
use async_trait::async_trait;
use tokio::sync::oneshot;

use crate::Status;

use super::{Dispatcher, DestinationDown, DispatcherType};



#[async_trait]
pub trait Producer<Out> {
    
    /// init used for initialize producer
    async fn init(&mut self);

    /// produce events, at maximum (max_demand)
    async fn handle_demand(&mut self, max_demand: usize) -> Vec<Out>;

    async fn terminate(&mut self);
}


// -----------------------------------------

pub struct ProducerRunnable<Out> {
    proc         : Box<dyn Producer<Out> + Send>,
    dispatcher   : Dispatcher<Out>,

    max_demand   : usize,
    shutdown     : oneshot::Receiver<()>

}



impl<Out> ProducerRunnable<Out> 
where
    Out: Clone + Send + 'static 
{
    pub fn new(proc: Box<dyn Producer<Out> + Send>,
               subscribe_to: Vec<Sender<Vec<Out>>>,
               dispatcher_type: Option<DispatcherType>,
               max_demand: usize,
               shutdown: oneshot::Receiver<()>)            
               
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
                    dispatcher,
                    max_demand,
                    shutdown,
                })        
            }
            Err(a) => {
                Err(a)
            }
        }
        
    }



    /// produce events and send to dst/subscribe_to by dispatcher
    #[inline]
    pub async fn produce_to_dst(&mut self) -> Result<(), DestinationDown<Out>> {
        let events = self.proc.handle_demand(self.max_demand).await;
        self.dispatcher.dispatch(events).await
    }


    #[inline]
    pub fn run(mut self) -> JoinHandle<Option<DestinationDown<Out>>> {
        
        tokio::spawn(async move {
            
            self.proc.init().await;

            loop {

                // If recv shutdown notify, call terminate   
                if let Ok(_) = self.shutdown.try_recv() {
                    self.proc.terminate().await;
                    return None
                }
                
                // produce events and dispatch
                match self.produce_to_dst().await {
                    Err(dd) => return Some(dd),
                    Ok(_) => (),
                }                
            }
        })
    }
}

