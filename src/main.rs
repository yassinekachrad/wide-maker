mod bybit;

use bybit::websocket::BybitWebsocket;
use config::Config;
use dotenv::dotenv;

use std::{process::exit, sync::{Arc, Mutex}, thread, time::Instant};
use serde_json::Value;
use rust_decimal::prelude::*;

use ctrlc;



const SUBSCRIPTION: &str = "orderbook.1.BTCUSDT";
const EDGE_BPS: i64 = 10; // 0.001

struct SharedMarketState {
    best_bid: Decimal,
    best_ask: Decimal,
}

fn main() {
    dotenv().ok();

    let settings = Config::builder()
        .add_source(config::File::with_name("config.json"))
        .add_source(config::Environment::with_prefix("API"))
        .build()
        .unwrap();
    
    let api_key = settings.get_string("key").expect("Missing API_KEY");
    let api_secret = settings.get_string("secret").expect("Missing API_SECRET");
    let symbol = settings.get_string("symbol").expect("Missing symbol");
    let qty = settings.get_string("qty").expect("Missing quantity");
    let subscriptions = vec![SUBSCRIPTION.to_string()];

    let mut bybit_ws = BybitWebsocket::new(&api_key, &api_secret, None, Some(subscriptions));

    let best_bid_mtx = Arc::new(Mutex::new(Decimal::from_str("0").unwrap()));
    let best_ask_mtx = Arc::new(Mutex::new(Decimal::from_str("0").unwrap()));
    let best_bid_mtx1 = Arc::clone(&best_bid_mtx);
    let best_ask_mtx1 = Arc::clone(&best_ask_mtx);

    let api_key1 = api_key.clone();
    let api_secret1 = api_secret.clone();
    let symbol1 = symbol.clone();

    let shared_market_state = SharedMarketState {
        best_bid: Decimal::from_str("0").unwrap(),
        best_ask: Decimal::from_str("0").unwrap(),
    };

    let sms_mtx = Arc::new(Mutex::new(shared_market_state));
    let sms_mtx1 = Arc::clone(&sms_mtx);


    let _ = ctrlc::set_handler(move || {
        println!("Cancelling orders..."); 
        let bybit_client = bybit::rest::BybitClient::new(api_key.clone(), api_secret.clone());
        let _ = bybit_client.cancel_all_orders(&symbol);
        exit(0);
    });

    let strategy_thread = thread::spawn(move || {
        let edge = Decimal::new(EDGE_BPS, 4);

        let mut my_current_bid: Decimal = Decimal::from_str("0").unwrap();
        let mut my_current_ask: Decimal = Decimal::from_str("0").unwrap();

        let bybit_client = bybit::rest::BybitClient::new(api_key1, api_secret1);

        loop {
            let sms_guard = sms_mtx.lock().unwrap();
            let mid = (sms_guard.best_bid + sms_guard.best_ask) / Decimal::from(2);
            let my_mid = (my_current_bid + my_current_ask) / Decimal::from(2);
            if my_mid.is_zero() && !mid.is_zero() || !my_mid.is_zero() && (mid - my_mid).abs() / my_mid > Decimal::from(0) {
                let new_bid = mid * (Decimal::from(1) - edge);
                let new_ask = mid * (Decimal::from(1) + edge);

                match bybit_client.cancel_all_orders(&symbol1) {
                    Ok(_) => {},
                    Err(e) => println!("Error canceling orders: {}", e)
                    
                }

                match bybit_client.place_order(&symbol1, &new_bid.round_dp(1).to_string(), &qty, true) {
                    Ok(_) => {},
                    Err(e) => println!("Error placing order: {}", e)
                }

                match bybit_client.place_order(&symbol1, &new_ask.round_dp(1).to_string(), &qty, false) {
                    Ok(_) => {},
                    Err(e) => println!("Error placing order: {}", e)
                }

                my_current_bid = new_bid;
                my_current_ask = new_ask;
                //println!("Current bid: {}, Current ask: {}, Best bid: {}, Best ask: {}", my_current_bid, my_current_ask, *dec_bb.lock().unwrap(), *dec_ba.lock().unwrap());
            }
        }
    });

    let message_handler = move |msg: String| {

        let v: Value = serde_json::from_str(&msg).unwrap();
        if v["topic"].is_string() && v["topic"].as_str().unwrap() == "orderbook.1.BTCUSDT" {
            let start_marker = Instant::now();

            let best_bid = if &v["data"]["b"][0][1] != "0" {&v["data"]["b"][0][0]} else {&v["data"]["b"][1][0]};
            let best_ask = if &v["data"]["a"][0][1] != "0" {&v["data"]["a"][0][0]} else {&v["data"]["a"][1][0]};

            let mut sms_guard = sms_mtx1.lock().unwrap();
            sms_guard.best_bid = best_bid.as_str().map(|x| Decimal::from_str(x).unwrap()).unwrap_or(sms_guard.best_bid);
            sms_guard.best_ask = best_ask.as_str().map(|x| Decimal::from_str(x).unwrap()).unwrap_or(sms_guard.best_ask);

            let _dt_s = start_marker.elapsed();


            println!("Best bid: {}, Best ask: {}, TTT (ns): {:?} - Coherent: {}", sms_guard.best_bid, sms_guard.best_ask, _dt_s, sms_guard.best_bid < sms_guard.best_ask);
        }
    };

    thread::spawn(move || {
        bybit_ws.run(message_handler);
    });

    strategy_thread.join().unwrap();
}
