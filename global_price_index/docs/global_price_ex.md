# Simple Example of Global Price Index Calculation
Let's break this down with a concrete example:

Scenario:

Imagine we have price data from 3 exchanges:
- Binance:
    + Mid-price: $50,000
    + Timestamp: 10 seconds ago
- Kraken:
    + Mid-price: $50,500
    + Timestamp: 60 seconds ago
- Huobi:
    + Mid-price: $49,800
    + Timestamp: 120 seconds ago

Let's assume our decay_factor is set to 300 seconds (5 minutes).
## Step 1: Calculate the weight for each price
Using the formula: weight = e^(-time_diff/decay_factor)
- Binance: weight = e^(-10/300) = 0.9670 (96.7%)
- Kraken: weight = e^(-60/300) = 0.8187 (81.9%)
- Huobi: weight = e^(-120/300) = 0.6703 (67.0%)

## Step 2: Calculate the weighted sum and total weight
Weighted Sum:
- Binance: $50,000 × 0.9670 = $48,350
- Kraken: $50,500 × 0.8187 = $41,344
- Huobi: $49,800 × 0.6703 = $33,381

Total: $48,350 + $41,344 + $33,381 = $123,075

Total Weight:
0.9670 + 0.8187 + 0.6703 = 2.456

## Step 3: Calculate the final weighted average
Global Price Index = $123,075 ÷ 2.456 = $50,112

Comparison with Simple Average

If we had used a simple average instead:

Simple Average = ($50,000 + $50,500 + $49,800) ÷ 3 = $50,100

## Interpretation
In this example, the weighted average ($50,112) is slightly higher than the simple average ($50,100) because:
- The higher price from Kraken ($50,500) still has significant weight (81.9%)
- The most recent price from Binance ($50,000) has the highest weight (96.7%)
- The oldest price from Huobi ($49,800) has the lowest weight (67.0%)

This demonstrates how the time-based weighting gives more influence to recent prices while still considering older prices with reduced impact. If Binance's price had been much higher or lower than the others, it would have pulled the weighted average more strongly in that direction because it's the most recent.