
mod stock;



fn main() {
    stock::run_simulation();
    //stock::benchmarkmarco();

}




use std::time::{Duration, Instant};
use stock::run_simulation;

//fn main() {
    //let duration = Duration::new(60, 0); 
    //let mut count = 0;
    //let start_time = Instant::now();

    //while Instant::now() - start_time < duration {
        //stock::run_simulation(); 
       //count += 1;
    //}

    //println!("Number of executions in 60 second: {}", count);
//}
