# Binance WebSocket Client

This project is a comprehensive WebSocket client designed to interact with the Binance API. It allows users to subscribe to various data streams such as `aggTrade`, `trade`, and `kline`. The client processes real-time data, provides a graphical user interface (GUI) to display the data, and includes features for custom stream subscriptions and graceful shutdown handling.

## Table of Contents

- [Features](#features)
- [Requirements](#requirements)
- [Installation](#installation)
- [Usage](#usage)
   - [Menu Options](#menu-options)
- [Project Structure](#project-structure)
- [Running Tests](#running-tests)
- [License](#license)

## Features

- **Real-time Data Processing**: Subscribe to and process real-time data streams from Binance.
- **Graphical User Interface**: Utilizes `tui-rs` to display trade data and statistics in a user-friendly interface.
- **Custom Stream Subscriptions**: Allows subscribing to multiple custom streams for flexibility.
- **Graceful Shutdown Handling**: Captures shutdown signals to exit the application cleanly.
- **Comprehensive Statistics**: Calculates and displays average price, median price, standard deviation, total volume, and volume-weighted average price (VWAP).
- **Modular Design**: Organized into several modules to enhance modularity and maintainability.

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
- **List Subscriptions**: List current subscriptions.
- **Exit**: Exit the application.

## Project Structure

The project is organized into several modules to enhance modularity and maintainability:

- **client**: Contains the main WebSocket client logic, including running the client, handling subscriptions, and managing shutdown.
- **handler**: Includes handlers for different types of messages (e.g., aggTrade) and input handling for graceful shutdown.
- **input**: Manages user input for shutdown signals.
- **message**: Parses incoming WebSocket messages.
- **ping**: Manages periodic pings to keep the WebSocket connection alive.
- **shutdown**: Handles graceful shutdown on receiving a shutdown signal.
- **storage**: Manages storage and processing of trade data.
- **subscriptions**: Manages subscription messages to the Binance WebSocket API.
- **ui**: Renders the user interface using `tui-rs`.

## Running Tests

The project includes unit tests for various components. To run the tests, use the following command:
```sh
cargo test
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
