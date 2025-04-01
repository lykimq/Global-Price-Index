interface ExchangePrice {
  exchange: string;
  mid_price: number;
}

interface PriceData {
  price: number;
  timestamp: string;
  exchange_prices: ExchangePrice[];
}

class PriceDisplay {
  private lastPrices: { [key: string]: number } = {};

  private formatPrice(price: number): string {
    return new Intl.NumberFormat("en-US", {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    }).format(price);
  }

  private formatTimestamp(timestamp: string): string {
    return new Date(timestamp).toLocaleTimeString();
  }

  private updateGlobalPrice(data: PriceData): void {
    const globalPriceEl = document.getElementById("global-price");
    const globalTimestampEl = document.getElementById("global-timestamp");

    if (globalPriceEl && globalTimestampEl) {
      if (this.lastPrices.global !== data.price) {
        globalPriceEl.className = `text-3xl font-bold ${
          data.price > this.lastPrices.global ? "price-up" : "price-down"
        }`;
      }
      globalPriceEl.textContent = `$${this.formatPrice(data.price)}`;
      globalTimestampEl.textContent = `Updated: ${this.formatTimestamp(
        data.timestamp
      )}`;
    }
  }

  private updateExchangePrices(data: PriceData): void {
    const exchangePricesEl = document.getElementById("exchange-prices");
    if (!exchangePricesEl) return;

    exchangePricesEl.innerHTML = data.exchange_prices
      .map((exchange) => {
        const priceClass =
          this.lastPrices[exchange.exchange] !== exchange.mid_price
            ? exchange.mid_price > this.lastPrices[exchange.exchange]
              ? "price-up"
              : "price-down"
            : "";
        this.lastPrices[exchange.exchange] = exchange.mid_price;

        return `
                <div class="flex justify-between items-center">
                    <span class="text-gray-600">${exchange.exchange}</span>
                    <span class="font-semibold ${priceClass}">$${this.formatPrice(
          exchange.mid_price
        )}</span>
                </div>
            `;
      })
      .join("");
  }

  private updateLastUpdateTime(timestamp: string): void {
    const lastUpdateEl = document.getElementById("last-update");
    if (lastUpdateEl) {
      lastUpdateEl.textContent = this.formatTimestamp(timestamp);
    }
  }

  public async updatePrices(): Promise<void> {
    try {
      const response = await fetch("/global-price");
      const data: PriceData = await response.json();

      this.updateGlobalPrice(data);
      this.updateExchangePrices(data);
      this.updateLastUpdateTime(data.timestamp);
    } catch (error) {
      console.error("Error fetching prices:", error);
    }
  }
}

// Initialize the price display
const priceDisplay = new PriceDisplay();

// Initial update
priceDisplay.updatePrices();

// Update every 5 seconds
setInterval(() => priceDisplay.updatePrices(), 5000);
