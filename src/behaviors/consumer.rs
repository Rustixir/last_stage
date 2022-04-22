

use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::channel;
use async_trait::async_trait;

use super::DestinationDown;

pub enum State<ConsumerIn> {
    Continue,
    Terminate,
    DestinationDown(Vec<ConsumerIn>)
}


#[async_trait]
pub trait Consumer<ConsumerIn> {
    
    /// init used for initialize producer
    async fn init(&mut self);

    /// receive events from upstream and cunsome it
    async fn handle_events(&mut self, upstream_events: Vec<ConsumerIn>) -> State<ConsumerIn>;


    async fn terminate(&mut self);
}

// -----------------------------------------

pub struct ConsumerRunnable<ConsumerIn> {
    proc         : Box<dyn Consumer<ConsumerIn> + Send>
}


impl<ConsumerIn> ConsumerRunnable<ConsumerIn> 
where
    ConsumerIn:  Clone + Send + 'static
{
    pub fn new(proc: Box<dyn Consumer<ConsumerIn> + Send> ) -> Self {
        ConsumerRunnable {
            proc
        }
    }


    #[inline]
    pub fn run(mut self, buffer: usize) -> Sender<Vec<ConsumerIn>> {

        let (sx, mut rx) = channel(buffer);

        tokio::spawn(async move {
            
            self.proc.init().await;

            loop {
            
                // Listen on channel
                match rx.recv().await {
                    Some(upstream_events) => {

                        // produce events and dispatch
                        match self.proc.handle_events(upstream_events).await {
                            State::Continue => (),
                            State::Terminate => {

                                // close channel to not get anymore 
                                rx.close();
                            }
                            State::DestinationDown(events) => {
                                return Some(DestinationDown(events))
                            }
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

