/**
 * Type definitions for the exchange price data structure.
 * Contains information about a single exchange's price.
 */
interface ExchangePrice {
  exchange: string;      // The name of the exchange (e.g., "Binance", "Kraken")
  mid_price: number;     // The mid-price from this exchange
}

/**
 * Type definition for the complete price data returned by the API.
 * This represents the structure of the /global-price endpoint response.
 */
interface PriceData {
  price: number;                     // The global weighted average price
  timestamp: string;                 // Timestamp when the price was calculated
  exchange_prices: ExchangePrice[];  // Array of individual exchange prices
}

/**
 * PriceDisplay class handles all UI updates and price formatting.
 * It's responsible for fetching price data and updating the DOM.
 */
class PriceDisplay {
  /**
   * Stores the last known prices for each exchange and the global price.
   * Used to determine if a price has increased or decreased for visual feedback.
   */
  private lastPrices: { [key: string]: number } = {};

  /**
   * Formats a number as a price with 2 decimal places.
   * price - The price to format
   * returns formatted price string (e.g., "50,432.12")
   */
  private formatPrice(price: number): string {
    return new Intl.NumberFormat("en-US", {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    }).format(price);
  }

  /**
   * Converts an ISO timestamp to a human-readable time format.
   * timestamp - ISO timestamp from the API
   * returns formatted time string (e.g., "3:45:22 PM")
   */
  private formatTimestamp(timestamp: string): string {
    return new Date(timestamp).toLocaleTimeString();
  }

  /**
   * Updates the global price section in the UI.
   * Applies visual cues (colors) for price changes.
   * data - Complete price data from the API
   */
  private updateGlobalPrice(data: PriceData): void {
    const globalPriceEl = document.getElementById("global-price");
    const globalTimestampEl = document.getElementById("global-timestamp");

    if (globalPriceEl && globalTimestampEl) {
      // Apply visual feedback (green/red) based on price movement
      if (this.lastPrices.global !== data.price) {
        globalPriceEl.className = `text-3xl font-bold ${data.price > this.lastPrices.global ? "price-up" : "price-down"
          }`;
      }

      // Update the displayed price and timestamp
      globalPriceEl.textContent = `$${this.formatPrice(data.price)}`;
      globalTimestampEl.textContent = `Updated: ${this.formatTimestamp(
        data.timestamp
      )}`;

      // Store current price for future comparison
      this.lastPrices.global = data.price;
    }
  }

  /**
   * Updates the exchange prices section in the UI.
   * Shows each exchange's price with visual cues for changes.
   * data - Complete price data from the API
   */
  private updateExchangePrices(data: PriceData): void {
    const exchangePricesEl = document.getElementById("exchange-prices");
    if (!exchangePricesEl) return;

    // Generate HTML for each exchange's price
    exchangePricesEl.innerHTML = data.exchange_prices
      .map((exchange) => {
        // Determine if price has changed and apply appropriate CSS class
        const priceClass =
          this.lastPrices[exchange.exchange] !== exchange.mid_price
            ? exchange.mid_price > this.lastPrices[exchange.exchange]
              ? "price-up"   // Price increased (green)
              : "price-down" // Price decreased (red)
            : "";            // No change

        // Store current price for future comparison
        this.lastPrices[exchange.exchange] = exchange.mid_price;

        // Return HTML for this exchange price
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

  /**
   * Updates the "Last Update" timestamp in the status section.
   */
  private updateLastUpdateTime(timestamp: string): void {
    const lastUpdateEl = document.getElementById("last-update");
    if (lastUpdateEl) {
      lastUpdateEl.textContent = this.formatTimestamp(timestamp);
    }
  }

  /**
   * Main method to fetch the latest price data and update the UI.
   * Calls the /global-price API endpoint and handles the response.
   * @returns Promise that resolves when the UI is updated
   */
  public async updatePrices(): Promise<void> {
    try {
      // Fetch the latest price data from the API
      const response = await fetch("/global-price");
      const data: PriceData = await response.json();

      // Update all sections of the UI with the new data
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

// Initial update to show prices immediately when page loads
priceDisplay.updatePrices();

// Set up automatic refresh every 5 seconds to keep prices current
setInterval(() => priceDisplay.updatePrices(), 5000);
