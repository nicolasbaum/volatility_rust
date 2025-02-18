# ETH/USDC Volatility Estimator

A real-time volatility estimator for the ETH/USDC pair that primarily uses Binance data, with optional support for Uniswap V3 on-chain data.

## Features

### Real-time Price Collection
- **Binance WebSocket Integration**
  - Direct connection to Binance's WebSocket API
  - Real-time trade data streaming
  - Automatic connection management
  - Instant price updates
  - Efficient network usage through WebSocket protocol

### Rolling Volatility Calculation
- **6-Hour Window Implementation**
  - Dynamic price data queue
  - Automatic removal of outdated prices
  - Memory-efficient storage
  - Continuous window sliding
  - Statistical accuracy maintenance

### High-Frequency Updates
- **5-Second Update Frequency**
  - Regular price checks
  - Rapid volatility recalculation
  - Smooth data flow
  - Minimal latency
  - Real-time market monitoring

### Optional Uniswap Integration
- **Feature Flag System**
  - Compile-time feature selection
  - Zero overhead when disabled
  - Easy integration when needed
  - Flexible deployment options
- **On-chain Data Access**
  - Direct Ethereum network connection
  - Uniswap V3 pool interaction
  - Smart contract data reading
  - Price calculation from sqrt price

### Comprehensive Logging
- **Structured Logging System**
  - Timestamp-based log entries
  - Multiple log levels (INFO, DEBUG, ERROR)
  - Detailed error reporting
  - Connection status tracking
  - Price update confirmation

### Error Handling
- **Robust Recovery System**
  - Automatic WebSocket reconnection
  - Error classification
  - Graceful degradation
  - Service continuity
  - Detailed error messages


## Setup and Installation

1. **Prerequisites**
   - Rust (latest stable version)
   - An Ethereum RPC URL (e.g., from Infura)
   - Internet connection for Binance WebSocket

2. **Environment Configuration**
   Create a `.env` file in the project root:
   ```
   ETHEREUM_RPC_URL=https://mainnet.infura.io/v3/YOUR_INFURA_KEY
   UNISWAP_V3_POOL_ADDRESS=0x8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8
   BINANCE_WS_URL=wss://stream.binance.com:9443/ws/ethusdc@trade
   ```

3. **Build and Run**
   ```bash
   # Build the project
   cargo build --release

   # Run with logging enabled
   RUST_LOG=info cargo run --release
   ```

## Implementation Approach

### Data Sources

1. **On-chain: Uniswap V3**
   - Connects to ETH/USDC pool on Ethereum mainnet
   - Fetches current price using the `slot0` function
   - Converts sqrtPriceX96 to actual price

2. **Off-chain: Binance**
   - Establishes WebSocket connection to Binance
   - Subscribes to ETH/USDC trade stream
   - Processes real-time trade data

### Volatility Calculation

The volatility is calculated using a rolling window approach:
1. Maintains a 6-hour window of price data
2. Calculates logarithmic returns between consecutive prices:
   ```
   log_return = ln(price_t / price_t-1)
   ```
3. Computes standard deviation of returns:
   ```
   volatility = sqrt(Σ(log_return - mean)² / (n-1))
   ```
4. Updates every minute with latest price data

Key features:
- Automatic removal of data points older than 6 hours
- Handles missing data points gracefully
- Combines prices from both sources for better accuracy

## Dependencies

Main crates used:
- `tokio`: Async runtime for concurrent operations
- `web3`: Ethereum interaction for Uniswap data
- `tungstenite`: WebSocket client for Binance data
- `chrono`: Time handling and window calculations
- `serde`: Data serialization/deserialization
- `env_logger`: Structured logging functionality

## Output Example
```
[2024-XX-XX HH:MM:SS INFO] Starting ETH/USDC volatility estimator...
[2024-XX-XX HH:MM:SS INFO] Connecting to Ethereum node at https://mainnet.infura.io/v3/...
[2024-XX-XX HH:MM:SS INFO] Initializing price collectors...
[2024-XX-XX HH:MM:SS INFO] Starting main loop...
[2024-XX-XX HH:MM:SS INFO] Received price: $2345.67 from Aggregated
[2024-XX-XX HH:MM:SS INFO] Current volatility estimate: 2.3456%
```
## Error Handling

The implementation includes:
- Robust error handling for both data sources
- Detailed error logging with timestamps
- Graceful degradation if one source fails
- Automatic reconnection for WebSocket disconnections

## Future Improvements

1. **Enhanced Data Collection**
   - Add more DEX sources for price data
   - Implement redundant CEX connections
   - Add historical data support for gap filling

2. **Advanced Analytics**
   - Volume-weighted price aggregation
   - Multiple volatility timeframes
   - Volatility forecasting capabilities

3. **Performance Optimizations**
   - Parallel price fetching from sources
   - Optimized data structures for calculations
   - Caching layer for frequent requests

4. **Reliability Enhancements**
   - Health check mechanisms
   - Automated recovery procedures
   - Performance metrics collection


