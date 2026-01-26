# Price API Reference & Rate Limits

This document outlines the cryptocurrency price APIs used by fiddupay and their respective rate limits for free usage.

## API Priority Order

fiddupay uses multiple APIs in fallback order to ensure maximum reliability:

1. **CoinGecko** (Primary)
2. **CoinCap** (Secondary) 
3. **CoinPaprika** (Tertiary)
4. **CryptoCompare** (Quaternary)
5. **Binance** (Last resort - geo-restricted)
6. **Kraken** (Additional fallback for some currencies)

## API Rate Limits & Documentation

### 1. CoinGecko API
- **Free Tier**: 10-30 calls/minute (varies by global usage)
- **Documentation**: https://www.coingecko.com/en/api/documentation
- **Endpoint**: `https://api.coingecko.com/api/v3/simple/price`
- **Usage**: Primary price source for all currencies
- **Rate Limit Handling**: 429 status code when exceeded

### 2. CoinCap API  
- **Free Tier**: 200 calls/minute (no API key), 500 calls/minute (with API key)
- **Documentation**: https://coincapapi.mintlify.app/api-reference/limits-and-Usage
- **Endpoint**: `https://api.coincap.io/v2/assets/{asset_id}`
- **Usage**: Secondary fallback for all currencies
- **Rate Limit Handling**: 429 status code when exceeded

### 3. CoinPaprika API
- **Free Tier**: 20,000 calls/month (personal use only)
- **Documentation**: https://coinpaprika-f4fde1b0.mintlify.app/api-plans
- **Endpoint**: `https://api.coinpaprika.com/v1/tickers/{coin_id}`
- **Usage**: Tertiary fallback for all currencies
- **Rate Limit Handling**: Status code monitoring

### 4. CryptoCompare API
- **Free Tier**: ~50 calls/minute
- **Documentation**: https://min-api.cryptocompare.com/documentation
- **Endpoint**: `https://min-api.cryptocompare.com/data/price`
- **Usage**: Quaternary fallback for all currencies
- **Rate Limit Handling**: Status code monitoring

### 5. Binance API
- **Free Tier**: Variable (geo-restricted in many locations)
- **Documentation**: https://binance-docs.github.io/apidocs/spot/en/#24hr-ticker-price-change-statistics
- **Endpoint**: `https://api.binance.com/api/v3/ticker/price`
- **Usage**: Last resort fallback
- **Limitations**: 451 status code in restricted regions

### 6. Kraken API
- **Free Tier**: No documented limits for ticker data
- **Documentation**: https://docs.kraken.com/rest/
- **Endpoint**: `https://api.kraken.com/0/public/Ticker`
- **Usage**: Additional fallback for SOL and ETH
- **Rate Limit Handling**: Status code monitoring

## Currency-Specific API Mappings

### Native Currencies
- **SOL**: `solana` (CoinGecko), `solana` (CoinCap), `sol-solana` (CoinPaprika)
- **ETH**: `ethereum` (CoinGecko), `ethereum` (CoinCap), `eth-ethereum` (CoinPaprika)
- **BNB**: `binancecoin` (CoinGecko), `binance-coin` (CoinCap), `bnb-binance-coin` (CoinPaprika)
- **ARB**: `arbitrum` (CoinGecko), `arbitrum` (CoinCap), `arb-arbitrum` (CoinPaprika)
- **MATIC**: `polygon-ecosystem-token` or `matic-network` (CoinGecko), `polygon` (CoinCap), `matic-polygon` (CoinPaprika)

### USDT Token Pricing Logic
USDT tokens use their underlying blockchain's native currency price:
- **USDT_ETH**: Uses ETH price
- **USDT_SPL**: Uses SOL price  
- **USDT_BEP20**: Uses BNB price
- **USDT_POLYGON**: Uses MATIC price
- **USDT_ARBITRUM**: Uses ARB price

## Error Handling & Fallback Strategy

1. **Rate Limit Detection**: Monitor for 429 status codes
2. **Automatic Fallback**: Switch to next API in priority order
3. **Timeout Handling**: 10-second timeout per API call
4. **Logging**: All API failures are logged with specific error details
5. **No Hardcoded Prices**: System fails gracefully if all APIs are unavailable

## Best Practices for Rate Limit Compliance

1. **Caching**: Prices are cached for 3 minutes to reduce API calls
2. **Sequential Fallback**: Only one API is called at a time
3. **Error Monitoring**: Failed API calls are logged for monitoring
4. **User Agent**: Proper user agent identification for API requests
5. **Timeout Management**: Reasonable timeouts prevent hanging requests

## Monitoring & Maintenance

- Monitor API response times and success rates
- Track rate limit violations in logs
- Consider upgrading to paid tiers for high-volume usage
- Regularly verify API endpoint availability
- Update currency mappings as needed

## Future Considerations

- **Paid Tiers**: Consider upgrading APIs for higher rate limits
- **Additional APIs**: Evaluate new price data sources
- **Caching Strategy**: Implement Redis-based price caching for scale
- **Load Balancing**: Distribute API calls across multiple keys/accounts
