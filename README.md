
# LastStage

<div align="center">

  <!-- Downloads -->
  <a href="https://crates.io/crates/laststage">
    <img src="https://img.shields.io/crates/d/laststage.svg?style=flat-square"
      alt="Download" />
  </a>
</div>

LastStage is a specification for exchanging events between producers and consumers.
with **push-based model**


## This project currently provides :

  * **Producer** produce events and dispatch to subscribers by Dispatcher 
                   (no input - one/many output)


  * **ProducerConsumer** it get events from upstream and dispatch to subscribers by Dispatcher 
                            (one/many input - one/many output)


  * **Consumer** it get events from upstream and consume it 
                   (one/many input - no output)


  * **Dispatcher** first get one-many subscriber then start to dispatch events by two mode (Broadcast / RoundRobin)



## Examples

Examples directory:

  * [Simple]  (https://github.com/Rustixir/last_stage/blob/master/examples/simple.rs)  
                              
  * [Multi]   (https://github.com/Rustixir/last_stage/blob/master/examples/multi.rs) 
                                       


# Installation
```
laststage = "1.0.0"

```



# Quick example

```rust

#[tokio::main]
async fn main() {

    let(shutdown_sender, shutdown_recv) = channel();


    // -----------------------------------
    //
    // Producer -> ProducerConsumer -> Consumer 
    //
    // ------------------------------------



    // Run Consumer
    let log_chan = ConsumerRunnable::new(Box::new(Log)).run(100);


    // Run ProducerConsumer
    let filter_chan = ProducerConsumerRunnable::new(Box::new(FilterByAge), 
                                                    vec![log_chan], 
                                                    Some(DispatcherType::RoundRobin)
                                                    ).unwrap().run(100);

    // Run Producer
    let _ = ProducerRunnable::new(Box::new(Prod), 
                                  vec![filter_chan], 
                                  None, 
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
    async fn init(&mut self) {}
    async fn terminate(&mut self) {}

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

} 


// -------------------------------------------


struct FilterByAge;

#[async_trait]
impl ProducerConsumer<ProdEvent, ProdEvent> for FilterByAge {
    async fn init(&mut self) {}
    async fn terminate(&mut self) {}

    async fn handle_events(&mut self, events: Vec<ProdEvent>) -> Vec<ProdEvent> {
        events
            .into_iter()
            .filter(|pe| pe.age > 25 && pe.age < 32)
            .collect()
    }



} 



struct Log;

#[async_trait]
impl Consumer<ProdEvent> for Log {
    async fn init(&mut self) {}
    async fn terminate(&mut self) {}

    async fn handle_events(&mut self, events: Vec<ProdEvent>) -> State<ProdEvent> {
        events
            .into_iter()
            .for_each(|pe| {
                println!("==> {} -> {}", pe.funame, pe.age)
            });
        
        State::Continue
    }  


}


```
