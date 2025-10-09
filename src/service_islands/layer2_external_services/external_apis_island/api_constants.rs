// API Constants Component
//
// This module contains all API URL constants used by the market data API.

/// API URLs - extracted from existing data_service.rs with cache-friendly grouping

// Binance APIs (Primary)
pub const BINANCE_BTC_PRICE_URL: &str = "https://api.binance.com/api/v3/ticker/24hr?symbol=BTCUSDT"; // 30 sec cache
pub const BINANCE_ETH_PRICE_URL: &str = "https://api.binance.com/api/v3/ticker/24hr?symbol=ETHUSDT"; // 30 sec cache
pub const BINANCE_SOL_PRICE_URL: &str = "https://api.binance.com/api/v3/ticker/24hr?symbol=SOLUSDT"; // 30 sec cache
pub const BINANCE_XRP_PRICE_URL: &str = "https://api.binance.com/api/v3/ticker/24hr?symbol=XRPUSDT"; // 30 sec cache
pub const BINANCE_ADA_PRICE_URL: &str = "https://api.binance.com/api/v3/ticker/24hr?symbol=ADAUSDT"; // 30 sec cache
pub const BINANCE_LINK_PRICE_URL: &str = "https://api.binance.com/api/v3/ticker/24hr?symbol=LINKUSDT"; // 30 sec cache

// CoinGecko APIs (Fallback)
pub const BASE_GLOBAL_URL: &str = "https://api.coingecko.com/api/v3/global"; // 30 sec cache
pub const BASE_BTC_PRICE_URL: &str = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd&include_24hr_change=true"; // 30 sec cache

// CoinMarketCap APIs (Fallback)
pub const CMC_GLOBAL_URL: &str = "https://pro-api.coinmarketcap.com/v1/global-metrics/quotes/latest"; // 30 sec cache
pub const CMC_BTC_PRICE_URL: &str = "https://pro-api.coinmarketcap.com/v1/cryptocurrency/quotes/latest?symbol=BTC"; // 30 sec cache

// Other APIs
pub const BASE_FNG_URL: &str = "https://api.alternative.me/fng/?limit=1"; // 5 min cache
pub const BASE_RSI_URL_TEMPLATE: &str = "https://api.taapi.io/rsi?secret={secret}&exchange=binance&symbol=BTC/USDT&interval=1d"; // 5 min cache