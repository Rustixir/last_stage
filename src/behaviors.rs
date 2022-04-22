


use tokio::sync::mpsc::Sender;

pub mod producer;
pub mod consumer;
pub mod producer_consumer;



#[derive(Debug)]
pub enum Status {
    SenderNotFound,
    SendersRepetive
}


/// Vec<Out> is events produced but not exit any channel to consume it 
pub struct DestinationDown<Out>(Vec<Out>);





pub enum DispatcherType {
    RoundRobin,
    Broadcast
}


struct Dispatcher<Out> {
    c: usize,
    subscribe_to: Vec<Sender<Vec<Out>>>,
    dispatcher_type: DispatcherType
}

impl<Out> Dispatcher<Out> 
where
    Out: Clone + Send
{
    

    pub fn new(subscribe_to: Vec<Sender<Vec<Out>>>, 
               dispatcher_type: DispatcherType) -> Result<Self, Status> {

        // Check destinations to not be repetive
        if let Err(_) = Dispatcher::check(&subscribe_to) {
            return Err(Status::SendersRepetive);
        }

        Ok(Dispatcher { 
            c: 0, 
            subscribe_to,
            dispatcher_type
        })
    }


    #[inline]
    pub async fn dispatch(&mut self, events: Vec<Out>) -> Result<(), DestinationDown<Out>> {
        match self.dispatcher_type {
            DispatcherType::RoundRobin => {
                return self.roundrobin(events).await
            }
            DispatcherType::Broadcast => {
                self.broadcast(events).await;
                Ok(())   
            }
        }
    }




    
    #[inline]
    async fn broadcast(&mut self, events: Vec<Out>) {
        for index in 0..(self.subscribe_to.len() - 1) {
            let msg = events.clone();
            let _ = self.subscribe_to[index].send(msg).await;
        }

        let _ = self.subscribe_to[self.subscribe_to.len() - 1].send(events).await;

    }


    /// send events to next destination
    /// 
    /// roundrobin is safe if a destination terminate
    /// auto detect it and remove from destinations 
    #[inline]
    async fn roundrobin(&mut self, mut events: Vec<Out>) -> Result<(), DestinationDown<Out>> {
        let mut reason = Ok(());
        

        // ------------------------------------------
        //
        //  almost always a destination must never 
        //   closed unless want shutdown earth  or terminate a flow 
        //   this service,
        // 
        //     with this imagine, almost always
        //     this fn done with First try
        //
        //     so tried to prevent loop instruction for performance
        //
        // ------------------------------------------




        // --------------First Try--------------------

        // get next index
        let mut index = self.next_index();

        // send events 
        match self.subscribe_to[index].send(events).await {
            
            // sending was successful 
            Ok(_ok) => {

                // return Ok() 
                return reason
            }
            
            // channel closed
            Err(err) => {

                // take ownership of events
                events = err.0;

                // remove this sender from subscribe_to
                self.subscribe_to.remove(index);
                
                // if not exist destination return Err
                if self.subscribe_to.len() == 0 {

                    reason = Err(DestinationDown(events));
                    return reason
                }
            }
        }
        
        
        // when First try fail then 
        //  it run a loop for other destination try 
        //
        // -------------- Loop Try --------------------
        
        loop {
            // get next index
            index = self.next_index();

            // send events 
            match self.subscribe_to[index].send(events).await {
                
                // sending was successful 
                Ok(_ok) => {

                    // return Ok() 
                    return reason
                }
                
                // channel closed
                Err(err) => {

                    // take ownership of events
                    events = err.0;

                    // remove this sender from subscribe_to
                    self.subscribe_to.remove(index);
                    
                    
                    // if not exist destination return Err
                    if self.subscribe_to.len() == 0 {

                        reason = Err(DestinationDown(events));
                        return reason
                    }
                }
            }
        }
    }





    fn next_index(&mut self) -> usize {
        let mut index = self.c;

        self.c += 1;

        if index >= self.subscribe_to.len() {
            self.c = 0;
            index = 0;
        }

        return index
    }

    
    /// Check destinations to not be repetive
    fn check(subscribe_to: &Vec<Sender<Vec<Out>>>) -> Result<(), ()> {
        for (oindex, outer_dst) in subscribe_to.iter().enumerate() {
            for (iindex, inner_dst) in subscribe_to.iter().enumerate() {
            
                // if not was itself && channel was same
                if oindex != iindex && outer_dst.same_channel(inner_dst) {
                    return Err(())
                }
            
            }
        }

        Ok(())
    }
}


