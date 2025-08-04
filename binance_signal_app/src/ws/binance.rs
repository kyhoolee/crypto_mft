use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt};
use serde::Deserialize;
use std::{collections::{BTreeMap}, time::Duration};
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize)]
struct DepthUpdate {
    lastUpdateId: Option<u64>,
    bids: Vec<[String; 2]>,
    asks: Vec<[String; 2]>,
}

#[derive(Debug, Clone)]
pub struct OrderbookSnapshot {
    pub timestamp: DateTime<Utc>,
    pub bids: BTreeMap<f64, f64>,
    pub asks: BTreeMap<f64, f64>,
}

#[derive(Debug, Clone)]
pub struct BinanceOrderbookWS {
    pub symbol: String,
    pub depth_level: usize,
    pub orderbook: Arc<Mutex<OrderbookSnapshot>>,
}

impl BinanceOrderbookWS {
    pub fn new(symbol: &str, depth_level: usize) -> Self {
        Self {
            symbol: symbol.to_lowercase(),
            depth_level,
            orderbook: Arc::new(Mutex::new(OrderbookSnapshot {
                timestamp: Utc::now(),
                bids: BTreeMap::new(),
                asks: BTreeMap::new(),
            })),
        }
    }

    pub async fn start(self: Arc<Self>) {
        let url = format!(
            "wss://stream.binance.com:9443/ws/{}@depth{}@100ms",
            self.symbol, self.depth_level
        );

        loop {
            match connect_async(&url).await {
                Ok((ws_stream, _)) => {
                    println!("📡 Connected to Binance WS for {}", self.symbol);
                    let (_, mut read) = ws_stream.split();

                    while let Some(msg) = read.next().await {
                        if let Ok(Message::Text(text)) = msg {
                            if let Ok(data) = serde_json::from_str::<DepthUpdate>(&text) {
                                self.process_snapshot(data).await;
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("⚠️ WS Error: {:?}, reconnecting...", e);
                    tokio::time::sleep(Duration::from_secs(3)).await;
                }
            }
        }
    }

    async fn process_snapshot(&self, data: DepthUpdate) {
        let mut ob = self.orderbook.lock().await;
        ob.bids.clear();
        ob.asks.clear();

        for [price, qty] in data.bids {
            let p: f64 = price.parse().unwrap_or(0.0);
            let q: f64 = qty.parse().unwrap_or(0.0);
            if q > 0.0 {
                ob.bids.insert(p, q);
            }
        }

        for [price, qty] in data.asks {
            let p: f64 = price.parse().unwrap_or(0.0);
            let q: f64 = qty.parse().unwrap_or(0.0);
            if q > 0.0 {
                ob.asks.insert(p, q);
            }
        }

        ob.timestamp = Utc::now();
    }

    pub async fn get_best_price(&self) -> Option<((f64, f64), (f64, f64))> {
        let ob = self.orderbook.lock().await;
        let best_bid = ob.bids.iter().rev().next().map(|(p, q)| (*p, *q));
        let best_ask = ob.asks.iter().next().map(|(p, q)| (*p, *q));
        match (best_bid, best_ask) {
            (Some(bid), Some(ask)) => Some((bid, ask)),
            _ => None,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_parse_and_print_orderbook() {
        let raw = r#"
        {
            "lastUpdateId": 123456789,
            "bids": [["1.23", "100.0"], ["1.22", "90.0"]],
            "asks": [["1.25", "80.0"], ["1.26", "70.0"]]
        }
        "#;

        let parsed: DepthUpdate = serde_json::from_str(raw).unwrap();
        println!("Parsed bids: {:?}", parsed.bids);
        println!("Parsed asks: {:?}", parsed.asks);
        assert!(parsed.bids.len() > 0);
        assert!(parsed.asks.len() > 0);
    }

    #[tokio::test]
    async fn test_process_and_check_best_price() {
        let ob = BinanceOrderbookWS::new("btcusdt", 20);
        let raw = DepthUpdate {
            lastUpdateId: Some(42),
            bids: vec![["30100.1".into(), "1.5".into()], ["30099.9".into(), "0.5".into()]],
            asks: vec![["30101.2".into(), "0.8".into()], ["30102.0".into(), "1.0".into()]],
        };

        ob.process_snapshot(raw).await;

        if let Some(((bid_price, _), (ask_price, _))) = ob.get_best_price().await {
            println!("🟢 Best Bid: {:.2}, Best Ask: {:.2}", bid_price, ask_price);
            assert!(bid_price > 0.0);
            assert!(ask_price > bid_price); // kiểm tra không bị crossed
        } else {
            panic!("❌ Orderbook empty");
        }
    }

    #[tokio::test]
    async fn test_orderbook_timestamp_updated() {
        let ob = BinanceOrderbookWS::new("ethusdt", 10);
        let before = Utc::now();
        let raw = DepthUpdate {
            lastUpdateId: None,
            bids: vec![["2000.0".into(), "1.0".into()]],
            asks: vec![["2001.0".into(), "2.0".into()]],
        };

        ob.process_snapshot(raw).await;

        let snap = ob.orderbook.lock().await;
        println!("📅 Timestamp: {}", snap.timestamp);
        assert!(snap.timestamp >= before);
    }
}
