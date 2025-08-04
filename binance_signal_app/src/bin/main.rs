use binance_signal_app::ws::binance::BinanceOrderbookWS;

use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let pair = "cakebnb";
    let ob = Arc::new(BinanceOrderbookWS::new(pair, 20));
    let ob_clone = ob.clone();

    tokio::spawn(async move {
        ob_clone.start().await;
    });

    loop {
        if let Some(((bid_p, bid_q), (ask_p, ask_q))) = ob.get_best_price().await {
            println!(
                "🟢 Bid: {:.4} ({:.2}) | Ask: {:.4} ({:.2})",
                bid_p, bid_q, ask_p, ask_q
            );
        }
        sleep(Duration::from_secs(1)).await;
    }
}
