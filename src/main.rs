use std::process::exit;
use url::Url;
use tungstenite::connect;
use serde_json::json;

const WS_URL: &str = "wss://stream.bybit.com/v5/public/inverse";
const TOPIC: &str = "tickers.BTCUSD";

fn main() {
    let mut socket = match connect(Url::parse(WS_URL).unwrap()) {
        Ok(s) => s.0,
        Err(_) => {
            println!("Could not connect to {}", WS_URL);
            exit(1);
        }
    };

    let subscribe_message = json!({
            "op": "subscribe",
            "args": [
                TOPIC
            ]
    }).to_string();

    if socket.write_message(tungstenite::Message::text(subscribe_message)).is_err() {
        println!("Error sending subscribe message");
        exit(1);
    }

    let mut index_price: f64 = 0.0;
    let mut last_price: f64 = 0.0;

    loop {
        let message = match socket.read_message() {
            Ok(m) => m,
            Err(_) => {
                println!("Error reading message");
                exit(1);
            }
        };

        if !message.is_text() {
            println!("Unexpected message type");
            exit(1);
        }

        let parsed: serde_json::Value = serde_json::from_str(message.to_text().unwrap()).expect("Can't parse JSON");

        if parsed["success"].is_boolean() && !parsed["success"].as_bool().unwrap() {
            println!("Error subscribing to topic");
            exit(1);
        }

         if !(parsed["topic"].is_string() && parsed["topic"].as_str().unwrap() == TOPIC) {
            continue;
        }

        let data = parsed["data"].as_object().unwrap();
        if data.contains_key("indexPrice") {
             index_price = data.get("indexPrice").unwrap().as_str().unwrap().parse().expect("Expected index price");
        }

        if data.contains_key("lastPrice") {
            last_price = data.get("lastPrice").unwrap().as_str().unwrap().parse().expect("Expected last price");
        }

        if !(last_price > 0.0 && index_price > 0.0) {
            continue;
        }

        println!("\x1b[2J\x1b[H\x1b[?25l");   // clear screen, move cursor to top of screen, hide cursor
        println!("  {:+.2}", last_price - index_price);
    }
}
