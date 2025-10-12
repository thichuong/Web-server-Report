// Cryptocurrency Price Fetchers Component
//
// This module contains all cryptocurrency price fetching methods with fallback logic.

impl MarketDataApi {
    /// Fetch Bitcoin price with fallback chain
    pub async fn fetch_btc_price(&self) -> Result<serde_json::Value> {
        self.record_api_call();

        // Try Binance first
        match self.fetch_btc_price_binance().await {
            Ok(data) => {
                self.record_success();
                Ok(data)
            }
            Err(e) => {
                println!("⚠️ Binance BTC price failed: {}, trying CoinGecko...", e);
                // Fallback to CoinGecko
                match self.fetch_btc_price_coingecko().await {
                    Ok(data) => {
                        self.record_success();
                        Ok(data)
                    }
                    Err(cg_err) => {
                        println!("⚠️ CoinGecko BTC price also failed: {}, trying CoinMarketCap...", cg_err);
                        // Final fallback to CoinMarketCap
                        match self.fetch_btc_price_cmc().await {
                            Ok(data) => {
                                self.record_success();
                                Ok(data)
                            }
                            Err(cmc_err) => {
                                self.record_failure();
                                println!("❌ All three APIs failed for BTC price");
                                Err(anyhow::anyhow!("Primary error: {}. CoinGecko error: {}. CoinMarketCap error: {}", e, cg_err, cmc_err))
                            }
                        }
                    }
                }
            }
        }
    }

    /// Fetch Bitcoin price from Binance
    async fn fetch_btc_price_binance(&self) -> Result<serde_json::Value> {
        let result = self.fetch_with_retry(BINANCE_BTC_PRICE_URL, |response_data: BinanceBtcPrice| {
            // Parse price and change from strings
            let price_usd: f64 = response_data.last_price.parse().unwrap_or(0.0);
            let change_24h: f64 = response_data.price_change_percent.parse().unwrap_or(0.0);

            serde_json::json!({
                "price_usd": price_usd,
                "change_24h": change_24h,
                "source": "binance",
                "last_updated": chrono::Utc::now().to_rfc3339()
            })
        }).await?;

        // Post-processing validation: check if we got meaningful data
        let price_usd = result.get("price_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);

        // Critical validation: Bitcoin price must be positive and reasonable
        if price_usd <= 0.0 || price_usd > 1_000_000.0 { // Basic sanity check
            return Err(anyhow::anyhow!(
                "Binance Bitcoin price validation failed: price_usd={}",
                price_usd
            ));
        }

        Ok(result)
    }

    /// Fetch Bitcoin price from CoinGecko
    async fn fetch_btc_price_coingecko(&self) -> Result<serde_json::Value> {
        let result = self.fetch_with_retry(BASE_BTC_PRICE_URL, |response_data: CoinGeckoBtcPrice| {
            serde_json::json!({
                "price_usd": response_data.bitcoin.usd,
                "change_24h": response_data.bitcoin.usd_24h_change,
                "source": "coingecko",
                "last_updated": chrono::Utc::now().to_rfc3339()
            })
        }).await?;

        // Post-processing validation: check if we got meaningful data
        let price_usd = result.get("price_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);

        // Critical validation: Bitcoin price must be positive and reasonable
        if price_usd <= 0.0 || price_usd > 1_000_000.0 { // Basic sanity check
            return Err(anyhow::anyhow!(
                "CoinGecko Bitcoin price validation failed: price_usd={}",
                price_usd
            ));
        }

        Ok(result)
    }

    /// Fetch Bitcoin price from CoinMarketCap
    async fn fetch_btc_price_cmc(&self) -> Result<serde_json::Value> {
        let cmc_key = self.cmc_api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("CoinMarketCap API key not provided"))?;

        let mut attempts = 0;
        let max_attempts = 3;

        while attempts < max_attempts {
            let response = self.client
                .get(CMC_BTC_PRICE_URL)
                .header("X-CMC_PRO_API_KEY", cmc_key)
                .header("Accept", "application/json")
                .send()
                .await?;

            match response.status() {
                status if status.is_success() => {
                    let cmc_data: CmcBtcResponse = response.json().await?;

                    if let Some(btc_data) = cmc_data.data.get("BTC").and_then(|v| v.first()) {
                        if let Some(usd_quote) = btc_data.quote.get("USD") {
                            return Ok(serde_json::json!({
                                "price_usd": usd_quote.price,
                                "change_24h": usd_quote.percent_change_24h,
                                "source": "coinmarketcap",
                                "last_updated": chrono::Utc::now().to_rfc3339()
                            }));
                        }
                    }
                    return Err(anyhow::anyhow!("Invalid CoinMarketCap BTC response structure"));
                }
                status if status == 429 => {
                    attempts += 1;
                    if attempts >= max_attempts {
                        return Err(anyhow::anyhow!("CoinMarketCap rate limit exceeded after {} attempts", max_attempts));
                    }

                    let delay = std::time::Duration::from_millis(1000 * (2_u64.pow(attempts)));
                    println!("⚠️ CoinMarketCap rate limit (429), retrying in {:?} (attempt {}/{})", delay, attempts, max_attempts);
                    tokio::time::sleep(delay).await;
                    continue;
                }
                status => {
                    return Err(anyhow::anyhow!("CoinMarketCap BTC API returned status: {}", status));
                }
            }
        }

        Err(anyhow::anyhow!("CoinMarketCap BTC API max retry attempts reached"))
    }

    /// Fetch Ethereum price from Binance
    pub async fn fetch_eth_price(&self) -> Result<serde_json::Value> {
        self.record_api_call();

        let result = self.fetch_with_retry(BINANCE_ETH_PRICE_URL, |response_data: BinanceBtcPrice| {
            let price_usd: f64 = response_data.last_price.parse().unwrap_or(0.0);
            let change_24h: f64 = response_data.price_change_percent.parse().unwrap_or(0.0);

            serde_json::json!({
                "price_usd": price_usd,
                "change_24h": change_24h,
                "source": "binance",
                "last_updated": chrono::Utc::now().to_rfc3339()
            })
        }).await?;

        let price_usd = result.get("price_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);
        if price_usd <= 0.0 || price_usd > 100_000.0 {
            return Err(anyhow::anyhow!("Binance Ethereum price validation failed: price_usd={}", price_usd));
        }

        Ok(result)
    }

    /// Fetch Solana price from Binance
    pub async fn fetch_sol_price(&self) -> Result<serde_json::Value> {
        self.record_api_call();

        let result = self.fetch_with_retry(BINANCE_SOL_PRICE_URL, |response_data: BinanceBtcPrice| {
            let price_usd: f64 = response_data.last_price.parse().unwrap_or(0.0);
            let change_24h: f64 = response_data.price_change_percent.parse().unwrap_or(0.0);

            serde_json::json!({
                "price_usd": price_usd,
                "change_24h": change_24h,
                "source": "binance",
                "last_updated": chrono::Utc::now().to_rfc3339()
            })
        }).await?;

        let price_usd = result.get("price_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);
        if price_usd <= 0.0 || price_usd > 10_000.0 {
            return Err(anyhow::anyhow!("Binance Solana price validation failed: price_usd={}", price_usd));
        }

        Ok(result)
    }

    /// Fetch XRP price from Binance
    pub async fn fetch_xrp_price(&self) -> Result<serde_json::Value> {
        self.record_api_call();

        let result = self.fetch_with_retry(BINANCE_XRP_PRICE_URL, |response_data: BinanceBtcPrice| {
            let price_usd: f64 = response_data.last_price.parse().unwrap_or(0.0);
            let change_24h: f64 = response_data.price_change_percent.parse().unwrap_or(0.0);

            serde_json::json!({
                "price_usd": price_usd,
                "change_24h": change_24h,
                "source": "binance",
                "last_updated": chrono::Utc::now().to_rfc3339()
            })
        }).await?;

        let price_usd = result.get("price_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);
        if price_usd <= 0.0 || price_usd > 100.0 {
            return Err(anyhow::anyhow!("Binance XRP price validation failed: price_usd={}", price_usd));
        }

        Ok(result)
    }

    /// Fetch Cardano price from Binance
    pub async fn fetch_ada_price(&self) -> Result<serde_json::Value> {
        self.record_api_call();

        let result = self.fetch_with_retry(BINANCE_ADA_PRICE_URL, |response_data: BinanceBtcPrice| {
            let price_usd: f64 = response_data.last_price.parse().unwrap_or(0.0);
            let change_24h: f64 = response_data.price_change_percent.parse().unwrap_or(0.0);

            serde_json::json!({
                "price_usd": price_usd,
                "change_24h": change_24h,
                "source": "binance",
                "last_updated": chrono::Utc::now().to_rfc3339()
            })
        }).await?;

        let price_usd = result.get("price_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);
        if price_usd <= 0.0 || price_usd > 100.0 {
            return Err(anyhow::anyhow!("Binance Cardano price validation failed: price_usd={}", price_usd));
        }

        Ok(result)
    }

    /// Fetch Chainlink price from Binance
    pub async fn fetch_link_price(&self) -> Result<serde_json::Value> {
        self.record_api_call();

        let result = self.fetch_with_retry(BINANCE_LINK_PRICE_URL, |response_data: BinanceBtcPrice| {
            let price_usd: f64 = response_data.last_price.parse().unwrap_or(0.0);
            let change_24h: f64 = response_data.price_change_percent.parse().unwrap_or(0.0);

            serde_json::json!({
                "price_usd": price_usd,
                "change_24h": change_24h,
                "source": "binance",
                "last_updated": chrono::Utc::now().to_rfc3339()
            })
        }).await?;

        let price_usd = result.get("price_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);
        if price_usd <= 0.0 || price_usd > 1_000.0 {
            return Err(anyhow::anyhow!("Binance Chainlink price validation failed: price_usd={}", price_usd));
        }

        Ok(result)
    }

    /// Fetch Binance Coin price with fallback chain
    pub async fn fetch_bnb_price(&self) -> Result<serde_json::Value> {
        self.record_api_call();

        // Try Binance first
        match self.fetch_bnb_price_binance().await {
            Ok(data) => {
                self.record_success();
                Ok(data)
            }
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("418") {
                    println!("⚠️ Binance BNB price failed: API returned status: 418 I'm a teapot for URL: https://api.binance.com/api/v3/ticker/24hr?symbol=BNBUSDT, trying CoinGecko...");
                } else {
                    println!("⚠️ Binance BNB price failed: {}, trying CoinGecko...", e);
                }
                // Fallback to CoinGecko
                match self.fetch_bnb_price_coingecko().await {
                    Ok(data) => {
                        self.record_success();
                        Ok(data)
                    }
                    Err(cg_err) => {
                        println!("⚠️ CoinGecko BNB price also failed: {}, trying CoinMarketCap...", cg_err);
                        // Final fallback to CoinMarketCap
                        match self.fetch_bnb_price_cmc().await {
                            Ok(data) => {
                                self.record_success();
                                Ok(data)
                            }
                            Err(cmc_err) => {
                                self.record_failure();
                                println!("❌ All three APIs failed for BNB price");
                                Err(anyhow::anyhow!("Primary error: {}. CoinGecko error: {}. CoinMarketCap error: {}", e, cg_err, cmc_err))
                            }
                        }
                    }
                }
            }
        }
    }

    /// Fetch Binance Coin price from Binance
    async fn fetch_bnb_price_binance(&self) -> Result<serde_json::Value> {
        let result = self.fetch_with_retry(BINANCE_BNB_PRICE_URL, |response_data: BinanceBtcPrice| {
            // Parse price and change from strings
            let price_usd: f64 = response_data.last_price.parse().unwrap_or(0.0);
            let change_24h: f64 = response_data.price_change_percent.parse().unwrap_or(0.0);

            serde_json::json!({
                "price_usd": price_usd,
                "change_24h": change_24h,
                "source": "binance",
                "last_updated": chrono::Utc::now().to_rfc3339()
            })
        }).await?;

        // Post-processing validation: check if we got meaningful data
        let price_usd = result.get("price_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);

        // Critical validation: BNB price must be positive and reasonable
        if price_usd <= 0.0 || price_usd > 10_000.0 { // Basic sanity check
            return Err(anyhow::anyhow!(
                "Binance BNB price validation failed: price_usd={}",
                price_usd
            ));
        }

        Ok(result)
    }

    /// Fetch Binance Coin price from CoinGecko
    async fn fetch_bnb_price_coingecko(&self) -> Result<serde_json::Value> {
        let result = self.fetch_with_retry(BASE_BNB_PRICE_URL, |response_data: CoinGeckoBnbPrice| {
            serde_json::json!({
                "price_usd": response_data.binancecoin.usd,
                "change_24h": response_data.binancecoin.usd_24h_change,
                "source": "coingecko",
                "last_updated": chrono::Utc::now().to_rfc3339()
            })
        }).await?;

        // Post-processing validation: check if we got meaningful data
        let price_usd = result.get("price_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);

        // Critical validation: BNB price must be positive and reasonable
        if price_usd <= 0.0 || price_usd > 10_000.0 { // Basic sanity check
            return Err(anyhow::anyhow!(
                "CoinGecko BNB price validation failed: price_usd={}",
                price_usd
            ));
        }

        Ok(result)
    }

    /// Fetch Binance Coin price from CoinMarketCap
    async fn fetch_bnb_price_cmc(&self) -> Result<serde_json::Value> {
        let cmc_key = self.cmc_api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("CoinMarketCap API key not provided"))?;

        let mut attempts = 0;
        let max_attempts = 3;

        while attempts < max_attempts {
            let response = self.client
                .get(CMC_BNB_PRICE_URL)
                .header("X-CMC_PRO_API_KEY", cmc_key)
                .header("Accept", "application/json")
                .send()
                .await?;

            match response.status() {
                status if status.is_success() => {
                    let cmc_data: CmcBtcResponse = response.json().await?;

                    if let Some(bnb_data) = cmc_data.data.get("BNB").and_then(|v| v.first()) {
                        if let Some(usd_quote) = bnb_data.quote.get("USD") {
                            return Ok(serde_json::json!({
                                "price_usd": usd_quote.price,
                                "change_24h": usd_quote.percent_change_24h,
                                "source": "coinmarketcap",
                                "last_updated": chrono::Utc::now().to_rfc3339()
                            }));
                        }
                    }
                    return Err(anyhow::anyhow!("Invalid CoinMarketCap BNB response structure"));
                }
                status if status == 429 => {
                    attempts += 1;
                    if attempts >= max_attempts {
                        return Err(anyhow::anyhow!("CoinMarketCap rate limit exceeded after {} attempts", max_attempts));
                    }

                    let delay = std::time::Duration::from_millis(1000 * (2_u64.pow(attempts)));
                    println!("⚠️ CoinMarketCap rate limit (429), retrying in {:?} (attempt {}/{})", delay, attempts, max_attempts);
                    tokio::time::sleep(delay).await;
                    continue;
                }
                status => {
                    return Err(anyhow::anyhow!("CoinMarketCap BNB API returned status: {}", status));
                }
            }
        }

        Err(anyhow::anyhow!("CoinMarketCap BNB API max retry attempts reached"))
    }

    /// Generic fetch with retry logic and exponential backoff
    pub async fn fetch_with_retry<T, F>(&self, url: &str, transformer: F) -> Result<serde_json::Value>
    where
        T: for<'de> serde::Deserialize<'de>,
        F: Fn(T) -> serde_json::Value,
    {
        let mut attempts = 0;
        let max_attempts = 3;

        while attempts < max_attempts {
            let response = self.client
                .get(url)
                .header("Accept", "application/json")
                .send()
                .await?;

            match response.status() {
                status if status.is_success() => {
                    let data: T = response.json().await?;
                    return Ok(transformer(data));
                }
                status if status == 418 => {
                    // 418 I'm a teapot - Binance uses this for rate limiting/blocking
                    attempts += 1;
                    if attempts >= max_attempts {
                        return Err(anyhow::anyhow!("Binance blocked request (418 I'm a teapot) after {} attempts for URL: {}. This usually means rate limiting or IP blocking.", max_attempts, url));
                    }

                    let delay = std::time::Duration::from_millis(2000 * (2_u64.pow(attempts)));
                    println!("⚠️ Binance blocking (418) for {}, retrying in {:?} (attempt {}/{})", url, delay, attempts, max_attempts);
                    tokio::time::sleep(delay).await;
                    continue;
                }
                status if status == 429 => {
                    // Rate limiting - implement exponential backoff
                    attempts += 1;
                    if attempts >= max_attempts {
                        return Err(anyhow::anyhow!("Rate limit exceeded after {} attempts for URL: {}", max_attempts, url));
                    }

                    let delay = std::time::Duration::from_millis(1000 * (2_u64.pow(attempts)));
                    println!("⚠️ Rate limit (429) hit for {}, retrying in {:?} (attempt {}/{})", url, delay, attempts, max_attempts);
                    tokio::time::sleep(delay).await;
                    continue;
                }
                status => {
                    return Err(anyhow::anyhow!("API returned status: {} for URL: {}", status, url));
                }
            }
        }

        Err(anyhow::anyhow!("Max retry attempts reached for URL: {}", url))
    }
}