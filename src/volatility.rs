use std::collections::VecDeque;
use chrono::{Utc, Duration};
use crate::price_collector::PricePoint;

pub struct VolatilityCalculator {
    window_size: Duration,
    price_history: VecDeque<PricePoint>,
}

impl VolatilityCalculator {
    pub fn new(window_size: Duration) -> Self {
        Self {
            window_size,
            price_history: VecDeque::new(),
        }
    }

    pub fn add_price(&mut self, price: PricePoint) {
        self.price_history.push_back(price);
        
        // Remove old prices outside the window
        let cutoff = Utc::now() - self.window_size;
        while let Some(oldest) = self.price_history.front() {
            if oldest.timestamp < cutoff {
                self.price_history.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn calculate_volatility(&self) -> Option<f64> {
        if self.price_history.len() < 2 {
            return None;
        }

        // Calculate log returns
        let mut returns: Vec<f64> = Vec::new();
        let prices: Vec<_> = self.price_history.iter().collect();
        
        for i in 1..prices.len() {
            let log_return = (prices[i].price / prices[i-1].price).ln();
            returns.push(log_return);
        }

        // Calculate standard deviation
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / (returns.len() - 1) as f64;
        
        // Calculate annualized volatility
        // Get the actual average time between samples
        let time_diff = prices.last().unwrap().timestamp - prices.first().unwrap().timestamp;
        let actual_interval = time_diff.num_seconds() as f64 / (prices.len() - 1) as f64;
        let samples_per_year = (365.0 * 24.0 * 60.0 * 60.0) / actual_interval;
        
        let annualized_vol = variance.sqrt() * samples_per_year.sqrt();
        
        Some(annualized_vol)
    }
} 