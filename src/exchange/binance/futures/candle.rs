use barter_integration::model::{ SubscriptionId, instrument::Instrument, Exchange, Side};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::event::{MarketEvent, MarketIter};
use crate::exchange::binance::channel::BinanceChannel;
use crate::exchange::binance::futures::liquidation::BinanceLiquidation;
use crate::exchange::binance::trade::BinanceTrade;
use crate::exchange::ExchangeId;
use crate::exchange::subscription::ExchangeSub;
use crate::Identifier;
use crate::subscription::candle::{Candle, Candles};
use crate::subscription::liquidation::Liquidation;
use crate::subscription::Subscription;

/// [`BinanceFuturesUsd`](super::BinanceFuturesUsd) Kline || Candle message.
///
/// ### Raw Payload Examples
// {
// "e": "kline",     // Event type
// "E": 1638747660000,   // Event time
// "s": "BTCUSDT",    // Symbol
// "k": {
// "t": 1638747660000, // Kline start time
// "T": 1638747719999, // Kline close time
// "s": "BTCUSDT",  // Symbol
// "i": "1m",      // Interval
// "f": 100,       // First trade ID
// "L": 200,       // Last trade ID
// "o": "0.0010",  // Open price
// "c": "0.0020",  // Close price
// "h": "0.0025",  // High price
// "l": "0.0015",  // Low price
// "v": "1000",    // Base asset volume
// "n": 100,       // Number of trades
// "x": false,     // Is this kline closed?
// "q": "1.0000",  // Quote asset volume
// "V": "500",     // Taker buy base asset volume
// "Q": "0.500",   // Taker buy quote asset volume
// "B": "123456"   // Ignore
// }
// }
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct BinanceCandle {
    #[serde(alias ="s", deserialize_with = "de_kline_subscription_id")]
    pub subscription_id: SubscriptionId,
    #[serde(alias = "k")]
    pub kline: BinanceKline,
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct BinanceKline {
    #[serde(alias="t", deserialize_with = "barter_integration::de::de_u64_epoch_ms_as_datetime_utc")]
    pub start_time: DateTime<Utc>,
    #[serde(alias="T", deserialize_with = "barter_integration::de::de_u64_epoch_ms_as_datetime_utc")]
    pub close_time: DateTime<Utc>,
    #[serde(alias="s")]
    pub symbol: String,
    #[serde(alias="i")]
    pub interval: String,
    #[serde(alias="f")]
    pub first_trade_id: u64,
    #[serde(alias="L")]
    pub last_trade_id: u64,
    #[serde(alias="o", deserialize_with = "barter_integration::de::de_str")]
    pub open: f64,
    #[serde(alias="h",  deserialize_with = "barter_integration::de::de_str")]
    pub high: f64,
    #[serde(alias="l", deserialize_with = "barter_integration::de::de_str")]
    pub low: f64,
    #[serde(alias="c", deserialize_with = "barter_integration::de::de_str")]
    pub close: f64,
    #[serde(alias="v", deserialize_with = "barter_integration::de::de_str")]
    pub volume: f64,
    #[serde(alias="n")]
    pub num_trades: u64,
    #[serde(alias="x")]
    pub is_closed: bool,
    #[serde(alias="q", deserialize_with = "barter_integration::de::de_str")]
    pub quote_asset_volume: f64,
    #[serde(alias="V", deserialize_with = "barter_integration::de::de_str")]
    pub taker_base_asset_volume: f64,
    #[serde(alias="Q", deserialize_with = "barter_integration::de::de_str")]
    pub taker_quote_asset_volume: f64,
    // #[serde(alias="B")]
    // pub ignore: u,
}

impl Identifier<Option<SubscriptionId>> for BinanceCandle {
    fn id(&self) -> Option<SubscriptionId> {
        Some(self.subscription_id.clone())
    }
}

impl From<(ExchangeId, Instrument, BinanceCandle)> for MarketIter<Candle> {
    fn from(
        (exchange_id, instrument, candle): (ExchangeId, Instrument, BinanceCandle),
    ) -> Self {
        Self(vec![Ok(MarketEvent {
            exchange_time: candle.kline.start_time,
            received_time: Utc::now(),
            exchange: Exchange::from(exchange_id),
            instrument,
            kind: Candle {
                close_time: Default::default(),
                open: candle.kline.open,
                high: candle.kline.high,
                low: candle.kline.low,
                close: candle.kline.close,
                volume: candle.kline.volume,
                trade_count: candle.kline.num_trades,
            },
        })])
    }
}
/// Deserialize a [`BinanceCandle`] "s" (eg/ "BTCUSDT") as the associated [`SubscriptionId`]
/// (eg/ "@klinesBTCUSDT").
pub fn de_kline_subscription_id<'de, D>(deserializer: D) -> Result<SubscriptionId, D::Error>
    where
        D: serde::de::Deserializer<'de>,
{
    <&str as Deserialize>::deserialize(deserializer)
        .map(|market| ExchangeSub::from((BinanceChannel::CANDLES, market)).id())
}
