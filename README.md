# Binance WebSocket Client

This project is a WebSocket client for interacting with the Binance API. It allows subscribing to various streams such as `aggTrade`, `trade`, and `kline`.

## Table of Contents

- [Features](#features)
- [Requirements](#requirements)
- [Installation](#installation)
- [Usage](#usage)
- [Contributing](#contributing)
- [License](#license)

## Features

- Subscribe to `aggTrade`, `trade`, and `kline` streams.
- Custom stream subscriptions.
- Real-time data processing.
- Graceful shutdown handling.

## Requirements

- Rust (latest stable version)
- Cargo (latest stable version)

## Installation

1. Clone the repository:
    ```sh
    git clone https://github.com/pedrosfaria2/websocket_binance_api.git
    cd websocket_binance_api
    ```

2. Build the project:
    ```sh
    cargo build
    ```

3. Run the tests:
    ```sh
    cargo test
    ```

## Usage

1. Run the WebSocket client:
    ```sh
    cargo run
    ```

2. Follow the on-screen menu to subscribe to various streams.

### Menu Options

- **Subscribe to aggTrade**: Subscribe to aggregated trade data for a specific symbol.
- **Subscribe to trade**: Subscribe to trade data for a specific symbol.
- **Subscribe to kline**: Subscribe to kline (candlestick) data for a specific symbol and interval.
- **Custom Subscribe**: Subscribe to multiple custom streams.
- **List Symbols**: List all available symbols.
- **List Subscriptions**: List current subscriptions (placeholder).
- **Exit**: Exit the application.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
