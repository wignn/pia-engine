import { CandleData } from "@/types";

export function generateRealisticHistory(
  basePrice: number,
  count: number = 300,
  intervalSeconds: number = 900 // 15m default
): CandleData[] {
  const candles: CandleData[] = [];
  const now = Math.floor(Date.now() / 1000);
  let startTime = now - count * intervalSeconds;
  
  // Create realistic price trajectory (drop from 4520 to 4370 then consolidate like the image)
  let currentPrice = basePrice * 1.02; // start higher
  
  for (let i = 0; i < count; i++) {
    const time = startTime + i * intervalSeconds;
    
    // Simulate volatility and trend pattern
    const progress = i / count;
    let bias = 0;
    
    // sharp drop in middle 40% - 70%
    if (progress > 0.4 && progress < 0.75) {
      bias = -0.0018; // strong selloff
    } else if (progress >= 0.75) {
      bias = 0.0002; // slight recovery / consolidation
    } else {
      bias = -0.0003;
    }
    
    const volatility = currentPrice * 0.0025;
    const change = (Math.random() - 0.49 + bias) * volatility;
    
    const open = currentPrice;
    const close = open + change;
    const high = Math.max(open, close) + Math.random() * volatility * 0.7;
    const low = Math.min(open, close) - Math.random() * volatility * 0.7;
    const volume = Math.floor(Math.random() * 8000 + 1200);

    candles.push({
      time,
      open: Number(open.toFixed(3)),
      high: Number(high.toFixed(3)),
      low: Number(low.toFixed(3)),
      close: Number(close.toFixed(3)),
      volume,
    });

    currentPrice = close;
  }

  // Adjust last candle to match basePrice
  const diff = basePrice - candles[candles.length - 1].close;
  for (let j = 0; j < candles.length; j++) {
    const factor = j / candles.length;
    candles[j].open = Number((candles[j].open + diff * factor).toFixed(3));
    candles[j].high = Number((candles[j].high + diff * factor).toFixed(3));
    candles[j].low = Number((candles[j].low + diff * factor).toFixed(3));
    candles[j].close = Number((candles[j].close + diff * factor).toFixed(3));
  }

  return candles;
}
