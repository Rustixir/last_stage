
#[tokio::main]
async fn main() {

    let(shutdown_sender, shutdown_recv) = channel();


    // ------------------------------------
    //
    //            -----> FilterByAge   \
    //           /                      \
    // Producer / -----> FilterByAge     --> Consumer 
    //          \                       /
    //           \-----> FilterByAge   /
    //

    // --------------------------------------



    // Run Consumer
    let log_chan = ConsumerRunnable::new(Box::new(Log)).run(100);


    // Run ProducerConsumer
    let filter_chan1 = 
        ProducerConsumerRunnable::new(Box::new(FilterByAge), vec![log_chan.clone()], None).unwrap().run(100);

    // Run ProducerConsumer
    let filter_chan2 = 
        ProducerConsumerRunnable::new(Box::new(FilterByAge), vec![log_chan.clone()], None).unwrap().run(100);


    // Run ProducerConsumer
    let filter_chan3 = 
        ProducerConsumerRunnable::new(Box::new(FilterByAge), vec![log_chan.clone()], None).unwrap().run(100);


    // Run ProducerConsumer
    let filter_chan4 = 
        ProducerConsumerRunnable::new(Box::new(FilterByAge), vec![log_chan], None).unwrap().run(100);




    // Run Producer
    let _ = ProducerRunnable::new(Box::new(Prod), 
                                  vec![filter_chan1, filter_chan2, filter_chan3, filter_chan4], 
                                  Some(DispatcherType::RoundRobin), 
                                  100, 
                                  shutdown_recv).unwrap().run();

    
    tokio::time::sleep(Duration::from_secs(10)).await;
}


#[derive(Clone)]
struct ProdEvent {
    pub funame: String,
    pub age: i32
}

struct Prod;

#[async_trait]
impl Producer<ProdEvent> for Prod {
    async fn init(&mut self) {

    }

    async fn handle_demand(&mut self, max_demand: usize) -> Vec<ProdEvent> {
        (0..max_demand as i32)
            .into_iter()
            .map(|i| {
                
                ProdEvent { 
                    funame: format!("DanyalMh-{}", i), 
                    age: (i + 30) % 35 
                }

            })
            .collect()
    }

    async fn terminate(&mut self) {

    }
} 


// -------------------------------------------


struct FilterByAge;

#[async_trait]
impl ProducerConsumer<ProdEvent, ProdEvent> for FilterByAge {
    async fn init(&mut self) {
        
    }

    async fn handle_events(&mut self, events: Vec<ProdEvent>) -> Vec<ProdEvent> {
        events
            .into_iter()
            .filter(|pe| pe.age > 25 && pe.age < 32)
            .collect()
    }

    async fn terminate(&mut self) {

    }


} 



struct Log;

#[async_trait]
impl Consumer<ProdEvent> for Log {
    async fn init(&mut self) {

    }

    async fn handle_events(&mut self, events: Vec<ProdEvent>) -> State<ProdEvent> {
        events
            .into_iter()
            .for_each(|pe| {
                println!("==> {} -> {}", pe.funame, pe.age)
            });
        
        State::Continue
    }  


    async fn terminate(&mut self) {
        
    }
}