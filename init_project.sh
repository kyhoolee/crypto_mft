#!/bin/bash

PROJECT_NAME="binance_signal_app"

echo "🚀 Creating project: $PROJECT_NAME"
cargo new $PROJECT_NAME
cd $PROJECT_NAME

echo "📁 Creating module directories..."
mkdir -p src/{core,ws,web/static,alerts}
touch src/{config.rs,main.rs}
touch src/core/{candle.rs,orderbook.rs,signal.rs}
touch src/ws/binance.rs
touch src/alerts/{mod.rs,telegram.rs,discord.rs}
touch src/web/{mod.rs,api.rs}
echo "// UI static files will be placed here" > src/web/static/README.md

echo "🧱 Updating Cargo.toml dependencies..."
cat >> Cargo.toml <<EOF

# --- Dependencies for async & web ---
[dependencies]
tokio = { version = "1", features = ["full"] }
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["fs"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.11", features = ["json", "multipart", "gzip", "stream", "rustls-tls"] }
chrono = { version = "0.4", features = ["serde"] }
futures = "0.3"
tokio-tungstenite = "0.21"
url = "2"
open = "5"
EOF

echo "✅ Project structure initialized!"

echo ""
echo "📦 Next steps:"
echo "1. Implement WS client in src/ws/binance.rs"
echo "2. Implement candle aggregation in src/core/candle.rs"
echo "3. Build React UI separately and copy build/ to src/web/static/"
echo "4. Run the app using: cargo run"
