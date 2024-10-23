use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use crossbeam_channel::unbounded;
use rand::Rng;
use scheduled_thread_pool::ScheduledThreadPool;

#[derive(Debug, Clone)]
pub struct Stock {
    pub name: String,
    pub v: i32,
    pub prev_v: i32,
}

#[derive(Debug)]
pub struct Order {
    pub stock_name: String,
    pub order_type: String,
    pub quantity: i32,
    pub price: i32,
    pub prev_price: i32,
    pub reason: String,
    pub order_category: String,
}

impl Order {
    fn new(stock_name: String, order_type: String, quantity: i32, price: i32, prev_price: i32, reason: String, order_category: String) -> Self {
        Order {
            stock_name,
            order_type,
            quantity,
            price,
            prev_price,
            reason,
            order_category,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StockType {
    Tech,
    Food,
    Healthcare,
}

impl Stock {
    pub fn stock_type(&self) -> StockType {
        match self.name.as_str() {
            "AAPL" | "AMZN" | "GOOGL" | "MSFT" | "TSLA" | "FB" | "CRM" | "INTC" | "NVDA" |"WORK" | "FSLY" | "CRWD" |
            "DOCU" => StockType::Tech,
            "KO" | "PEP" | "MCD" | "SBUX" | "GIS" | "HSY" | "KR" | "CPB" | "WMT" |"TGT" | "COST" | "PG" | "UN" | "SYY" |
            "FLO" | "WBA" => StockType::Food,
            "MDLZ" | "MRK" | "AMGN" | "UNH" | "HCA" | "ANTM" |"DHR" | "ABT" |"TMO" |"REGN" | "ILMN" | "MDT" | "ZBH" | "VRTX" |
            "IDXX" |"DGX"  => StockType::Healthcare,
            _ => unreachable!(),
        }
    }
}

pub fn simulate_stock_changes(sched: &ScheduledThreadPool, shared_stock: Arc<Mutex<Vec<Stock>>>, stock_sel: crossbeam_channel::Sender<Stock>) {
    sched.execute_at_fixed_rate(
        Duration::from_micros(100),
        Duration::from_secs(1),
        move || {
            let mut rng = rand::thread_rng();
            let mut stocks = shared_stock.lock().unwrap();

            for stock in stocks.iter_mut() {
                stock.prev_v = stock.v;
                stock.v += rng.gen_range(-40..=60);
                println!("STOCK UPDATE: name: {}, v:{}", stock.name, stock.v);

                if stock_sel.send(stock.clone()).is_err() {
                    break;
                }
            }
        },
    );
}

pub fn process_broker_actions(
    name: String,
    broker_counts: Arc<HashMap<String, Mutex<i32>>>,
    sel_r: crossbeam_channel::Receiver<Stock>,
    client_preferences: HashMap<String, (StockType, String, i32, i32)>,
    transaction_limit: i32,
) -> JoinHandle<HashMap<String, i32>> {
    thread::spawn(move || {
        let mut client_transactions: HashMap<String, i32> = client_preferences.keys().map
        (|k| (k.clone(), 0)).collect();
        let mut client_earnings: HashMap<String, i32> = HashMap::new();

        while client_transactions.values().any(|&v| v < transaction_limit) {
            let stock = match sel_r.recv() {
                Ok(stock) => stock,
                Err(_) => break,
            };

            let price_change = stock.v - stock.prev_v;
            for (client_name, (stock_type, order_category, 
                min_change_buy, min_change_sell)) in &client_preferences {
                if stock.stock_type() != *stock_type || (*order_category != "Market" && 
                (price_change > -*min_change_buy && price_change < *min_change_sell)) {
                    continue;
                }

                let mut process_order = false;
                let mut order_type = "buying"; 
                let mut reason = String::new();
                if (*order_category == "Market" || price_change <= -*min_change_buy) && stock.v < stock.prev_v {
                    process_order = true;
                    reason = format!("Executed a buy due to price decrease to {}", stock.v);
                } else if (*order_category == "Market" || price_change >= *min_change_sell) && stock.v > stock.prev_v {
                    process_order = true;
                    order_type = "selling";
                    reason = format!("Executed a sell due to price increase to {}", stock.v);
                }

                if process_order {
                    let quantity = rand::thread_rng().gen_range(10..=100);

                    let order = Order::new(
                        stock.name.clone(),
                        order_type.to_string(),
                        quantity,
                        stock.v,
                        stock.prev_v,
                        reason,
                        order_category.to_string(),
                    );

                    if order_type == "selling" {
                        let earnings = quantity * (stock.v - stock.prev_v);
                        let client_earning = client_earnings.entry(client_name.clone()).or_insert(0);
                        *client_earning += earnings;
                    }

                    println!("{} for client {} placed a {} stock: {:?}", name, client_name, order_type, order);

                    let count = client_transactions.entry(client_name.clone()).or_insert(0);
                    *count += 1;

                    if *count >= transaction_limit {
                        continue;
                    }
                }
            }
        }

        println!("{} has completed the transactions for all clients.", name);
        client_earnings
    })
}





pub fn run_simulation() {
    println!("Stock updates from Bursa Malaysia...");
    let start = Instant::now();
    let sched = ScheduledThreadPool::new(5);
    let (sel_s, sel_r) = unbounded::<Stock>();
    let shared_stock = Arc::new(Mutex::new(vec![
        Stock { name: "AMZN".to_string(), v: 200, prev_v: 200 },
        Stock { name: "GOOGL".to_string(), v: 120, prev_v: 120 },
        Stock { name: "MSFT".to_string(), v: 130, prev_v: 130 },
        Stock { name: "TSLA".to_string(), v: 300, prev_v: 300 },
        Stock { name: "FB".to_string(), v: 156, prev_v: 156 },
        Stock { name: "CRM".to_string(), v: 90, prev_v: 90 },
        Stock { name: "INTC".to_string(), v: 245, prev_v: 245 },
        Stock { name: "NVDA".to_string(), v: 187, prev_v: 187 },
        Stock { name: "WORK".to_string(), v: 65, prev_v: 65 },
        Stock { name: "FSLY".to_string(), v: 110, prev_v: 110 },
        Stock { name: "CRWD".to_string(), v: 125, prev_v: 125 },
        Stock { name: "DOCU".to_string(), v: 240, prev_v: 240 },
        Stock { name: "NOW".to_string(), v: 180, prev_v: 180 },
        Stock { name: "PLTR".to_string(), v: 95, prev_v: 95 },
        Stock { name: "KO".to_string(), v: 310, prev_v: 310 },
        Stock { name: "PEP".to_string(), v: 400, prev_v: 400 },
        Stock { name: "MCD".to_string(), v: 170, prev_v: 170 },
        Stock { name: "SBUX".to_string(), v: 200, prev_v: 200 },
        Stock { name: "GIS".to_string(), v: 67, prev_v: 67 },
        Stock { name: "HSY".to_string(), v: 276, prev_v: 276 },
        Stock { name: "KR".to_string(), v: 22, prev_v: 22 },
        Stock { name: "CPB".to_string(), v: 120, prev_v: 120 },
        Stock { name: "PER".to_string(), v: 400, prev_v: 400 },
        Stock { name: "WMT".to_string(), v: 150, prev_v: 150 },
        Stock { name: "TGT".to_string(), v: 90, prev_v: 90 },
        Stock { name: "COST".to_string(), v: 280, prev_v: 280 },
        Stock { name: "PG".to_string(), v: 200, prev_v: 200 },
        Stock { name: "UN".to_string(), v: 170, prev_v: 170 },
        Stock { name: "SYY".to_string(), v: 110, prev_v: 110 },
        Stock { name: "FLO".to_string(), v: 30, prev_v: 30 },
        Stock { name: "WBA".to_string(), v: 55, prev_v: 55 },
        Stock { name: "MDLZ".to_string(), v: 330, prev_v: 330 },
        Stock { name: "MRK".to_string(), v: 280, prev_v: 280 },
        Stock { name: "AMGN".to_string(), v: 430, prev_v: 430 },
        Stock { name: "UNH".to_string(), v: 120, prev_v: 120 },
        Stock { name: "HCA".to_string(), v: 88, prev_v: 88 },
        Stock { name: "ANTM".to_string(), v: 22, prev_v: 22 },
        Stock { name: "DHR".to_string(), v: 120, prev_v: 120 },
        Stock { name: "ABT".to_string(), v: 400, prev_v: 400 },
        Stock { name: "TMO".to_string(), v: 150, prev_v: 150 },
        Stock { name: "REGN".to_string(), v: 90, prev_v: 90 },
        Stock { name: "ILMN".to_string(), v: 280, prev_v: 280 },
        Stock { name: "MDT".to_string(), v: 200, prev_v: 200 },
        Stock { name: "ZBH".to_string(), v: 170, prev_v: 170 },
        Stock { name: "VRTX".to_string(), v: 110, prev_v: 110 },
        Stock { name: "IDXX".to_string(), v: 30, prev_v: 30 },
        Stock { name: "DGX".to_string(), v: 55, prev_v: 55 },
        
    ]));

    let broker_count = Arc::new(HashMap::new());

    simulate_stock_changes(&sched, shared_stock.clone(), sel_s);

    let transaction_limit = 10; 

    let client_preferences_broker1 = HashMap::from([
        ("John".to_string(), (StockType::Tech, "Market".to_string(), 0, 0)),
        ("Peter".to_string(), (StockType::Tech, "Market".to_string(), 0, 0)),
    ]);

    let client_preferences_broker2 = HashMap::from([
        ("James".to_string(), (StockType::Food, "Limit".to_string(), 25, 40)),
    ]);

    let client_preferences_broker3 = HashMap::from([
        ("Alex".to_string(), (StockType::Healthcare, "Limit".to_string(), 10, 30)),
        ("Mike".to_string(), (StockType::Tech, "Market".to_string(), 0, 0)),
    ]);

    let broker1_thread = process_broker_actions(
        "Broker 1".to_string(), broker_count.clone(), sel_r.clone(), client_preferences_broker1, transaction_limit
    );
    let broker2_thread = process_broker_actions(
        "Broker 2".to_string(), broker_count.clone(), sel_r.clone(), client_preferences_broker2, transaction_limit
    );
    let broker3_thread = process_broker_actions(
        "Broker 3".to_string(), broker_count, sel_r, client_preferences_broker3, transaction_limit
    );

    let earnings_broker1 = broker1_thread.join().unwrap();
    let earnings_broker2 = broker2_thread.join().unwrap();
    let earnings_broker3 = broker3_thread.join().unwrap();

    let duration = Instant::now() - start;
    println!("Simulation ended. It took: {:?}", duration);

    // Final report
    println!("Final report:");
    println!("Broker 1 earnings:");
    for (client, earnings) in earnings_broker1 {
        println!("{} earned ${}", client, earnings);
    }
    println!("Broker 2 earnings:");
    for (client, earnings) in earnings_broker2 {
        println!("{} earned ${}", client, earnings);
    }
    println!("Broker 3 earnings:");
    for (client, earnings) in earnings_broker3 {
        println!("{} earned ${}", client, earnings);
    }
}

extern crate bma_benchmark;
use  bma_benchmark::{benchmark, staged_benchmark, staged_benchmark_print_for};
use core::hint::black_box;

pub fn benchmarkmarco() {
    staged_benchmark!("simulation", 30, {
        black_box(run_simulation());
    });
    staged_benchmark_print_for!("simulation")
}
