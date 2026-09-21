// FeatherSurf Benchmarking
//
// Reproducible benchmark harness for measuring browser performance.
// Tracks memory, CPU, startup time, and restore time.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

// ── Benchmark Configuration ────────────────────────────────────────

/// Hardware tier for benchmarking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HardwareTier {
    /// 4 GB RAM.
    Low,
    /// 8 GB RAM.
    Medium,
    /// 16 GB RAM.
    High,
}

impl HardwareTier {
    pub fn ram_bytes(&self) -> u64 {
        match self {
            HardwareTier::Low => 4 * 1024 * 1024 * 1024,
            HardwareTier::Medium => 8 * 1024 * 1024 * 1024,
            HardwareTier::High => 16 * 1024 * 1024 * 1024,
        }
    }
}

/// Benchmark type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BenchmarkType {
    /// Cold startup (no cache).
    ColdStartup,
    /// Warm startup (with cache).
    WarmStartup,
    /// Tab restore.
    TabRestore,
    /// Memory under load (10 tabs).
    MemoryLoad10,
    /// Memory under load (50 tabs).
    MemoryLoad50,
    /// Memory under load (100 tabs).
    MemoryLoad100,
    /// Background CPU usage.
    BackgroundCpu,
    /// Privacy filter latency.
    PrivacyFilter,
    /// Navigation speed.
    NavigationSpeed,
}

/// Benchmark result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkResult {
    /// Benchmark type.
    pub benchmark_type: BenchmarkType,
    /// Hardware tier.
    pub hardware_tier: HardwareTier,
    /// Platform.
    pub platform: String,
    /// Browser version.
    pub version: String,
    /// Timestamp.
    pub timestamp: u64,
    /// Metrics collected.
    pub metrics: HashMap<String, f64>,
    /// Whether the result met the budget.
    pub within_budget: bool,
}

/// Performance budget for a metric.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceBudget {
    /// Metric name.
    pub metric: String,
    /// Maximum allowed value.
    pub max_value: f64,
    /// Unit (ms, bytes, percent, etc.).
    pub unit: String,
}

// ── Benchmark Harness ──────────────────────────────────────────────

/// Benchmark harness for running and recording results.
pub struct BenchmarkHarness {
    /// Saved results.
    results: Vec<BenchmarkResult>,
    /// Performance budgets.
    budgets: HashMap<BenchmarkType, Vec<PerformanceBudget>>,
    /// Baseline results for comparison.
    baselines: HashMap<BenchmarkType, BenchmarkResult>,
}

impl BenchmarkHarness {
    pub fn new() -> Self {
        let mut harness = Self {
            results: Vec::new(),
            budgets: HashMap::new(),
            baselines: HashMap::new(),
        };

        // Set default budgets
        harness.set_budgets();
        harness
    }

    /// Set default performance budgets.
    fn set_budgets(&mut self) {
        // Cold startup: < 2000ms
        self.budgets.insert(BenchmarkType::ColdStartup, vec![
            PerformanceBudget { metric: "startup_ms".to_string(), max_value: 2000.0, unit: "ms".to_string() },
        ]);

        // Warm startup: < 500ms
        self.budgets.insert(BenchmarkType::WarmStartup, vec![
            PerformanceBudget { metric: "startup_ms".to_string(), max_value: 500.0, unit: "ms".to_string() },
        ]);

        // Tab restore: < 1000ms per tab
        self.budgets.insert(BenchmarkType::TabRestore, vec![
            PerformanceBudget { metric: "restore_ms".to_string(), max_value: 1000.0, unit: "ms".to_string() },
        ]);

        // Memory under load (10 tabs): < 300MB
        self.budgets.insert(BenchmarkType::MemoryLoad10, vec![
            PerformanceBudget { metric: "memory_bytes".to_string(), max_value: 300.0 * 1024.0 * 1024.0, unit: "bytes".to_string() },
        ]);

        // Memory under load (50 tabs): < 800MB
        self.budgets.insert(BenchmarkType::MemoryLoad50, vec![
            PerformanceBudget { metric: "memory_bytes".to_string(), max_value: 800.0 * 1024.0 * 1024.0, unit: "bytes".to_string() },
        ]);

        // Memory under load (100 tabs): < 1.5GB
        self.budgets.insert(BenchmarkType::MemoryLoad100, vec![
            PerformanceBudget { metric: "memory_bytes".to_string(), max_value: 1.5 * 1024.0 * 1024.0 * 1024.0, unit: "bytes".to_string() },
        ]);

        // Background CPU: < 1%
        self.budgets.insert(BenchmarkType::BackgroundCpu, vec![
            PerformanceBudget { metric: "cpu_percent".to_string(), max_value: 1.0, unit: "percent".to_string() },
        ]);

        // Privacy filter latency: < 1ms
        self.budgets.insert(BenchmarkType::PrivacyFilter, vec![
            PerformanceBudget { metric: "latency_ms".to_string(), max_value: 1.0, unit: "ms".to_string() },
        ]);

        // Navigation speed: < 500ms
        self.budgets.insert(BenchmarkType::NavigationSpeed, vec![
            PerformanceBudget { metric: "navigation_ms".to_string(), max_value: 500.0, unit: "ms".to_string() },
        ]);
    }

    /// Record a benchmark result.
    pub fn record(&mut self, result: BenchmarkResult) {
        // Check if within budget
        let within_budget = self.check_budget(&result);
        let mut result = result;
        result.within_budget = within_budget;

        self.results.push(result);
    }

    /// Check if a result is within budget.
    fn check_budget(&self, result: &BenchmarkResult) -> bool {
        if let Some(budgets) = self.budgets.get(&result.benchmark_type) {
            for budget in budgets {
                if let Some(&value) = result.metrics.get(&budget.metric) {
                    if value > budget.max_value {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Set a baseline result.
    pub fn set_baseline(&mut self, result: BenchmarkResult) {
        self.baselines.insert(result.benchmark_type, result);
    }

    /// Compare a result against baseline.
    pub fn compare_to_baseline(&self, result: &BenchmarkResult) -> Option<BenchmarkComparison> {
        let baseline = self.baselines.get(&result.benchmark_type)?;
        
        let mut comparisons = HashMap::new();
        for (metric, value) in &result.metrics {
            if let Some(&baseline_value) = baseline.metrics.get(metric) {
                let change = ((value - baseline_value) / baseline_value) * 100.0;
                comparisons.insert(metric.clone(), change);
            }
        }

        Some(BenchmarkComparison {
            baseline: baseline.clone(),
            current: result.clone(),
            changes: comparisons,
        })
    }

    /// Get all results.
    pub fn results(&self) -> &[BenchmarkResult] {
        &self.results
    }

    /// Get results by type.
    pub fn results_by_type(&self, benchmark_type: BenchmarkType) -> Vec<&BenchmarkResult> {
        self.results.iter().filter(|r| r.benchmark_type == benchmark_type).collect()
    }

    /// Get the latest result for a benchmark type.
    pub fn latest(&self, benchmark_type: BenchmarkType) -> Option<&BenchmarkResult> {
        self.results.iter()
            .filter(|r| r.benchmark_type == benchmark_type)
            .last()
    }

    /// Get budget for a benchmark type.
    pub fn budget(&self, benchmark_type: BenchmarkType) -> Option<&Vec<PerformanceBudget>> {
        self.budgets.get(&benchmark_type)
    }

    /// Set budget for a benchmark type.
    pub fn set_budget(&mut self, benchmark_type: BenchmarkType, budget: Vec<PerformanceBudget>) {
        self.budgets.insert(benchmark_type, budget);
    }

    /// Generate Markdown report.
    pub fn generate_report(&self) -> String {
        let mut report = String::from("# FeatherSurf Benchmark Report\n\n");
        report.push_str(&format!("Generated: {}\n\n", now_secs()));

        for (benchmark_type, _) in &self.budgets {
            let results = self.results_by_type(*benchmark_type);
            if results.is_empty() {
                continue;
            }

            report.push_str(&format!("## {:?}\n\n", benchmark_type));
            
            if let Some(latest) = self.latest(*benchmark_type) {
                let within = if latest.within_budget { "✅" } else { "❌" };
                report.push_str(&format!("Status: {}\n\n", within));
                
                report.push_str("| Metric | Value | Budget |\n");
                report.push_str("|--------|-------|--------|\n");
                
                if let Some(budgets) = self.budgets.get(benchmark_type) {
                    for budget in budgets {
                        if let Some(&value) = latest.metrics.get(&budget.metric) {
                            report.push_str(&format!(
                                "| {} | {:.2} {} | {} |\n",
                                budget.metric, value, budget.unit, budget.max_value
                            ));
                        }
                    }
                }
            }
            report.push_str("\n");
        }

        report
    }

    /// Clear all results.
    pub fn clear(&mut self) {
        self.results.clear();
    }
}

impl Default for BenchmarkHarness {
    fn default() -> Self {
        Self::new()
    }
}

// ── Benchmark Comparison ───────────────────────────────────────────

/// Comparison between current and baseline results.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkComparison {
    pub baseline: BenchmarkResult,
    pub current: BenchmarkResult,
    /// Percentage change for each metric (positive = worse).
    pub changes: HashMap<String, f64>,
}

// ── Helpers ────────────────────────────────────────────────────────

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_result() {
        let mut harness = BenchmarkHarness::new();
        let mut metrics = HashMap::new();
        metrics.insert("startup_ms".to_string(), 1500.0);

        harness.record(BenchmarkResult {
            benchmark_type: BenchmarkType::ColdStartup,
            hardware_tier: HardwareTier::Medium,
            platform: "windows".to_string(),
            version: "0.0.1".to_string(),
            timestamp: now_secs(),
            metrics,
            within_budget: true,
        });

        assert_eq!(harness.results().len(), 1);
    }

    #[test]
    fn check_budget() {
        let mut harness = BenchmarkHarness::new();
        let mut metrics = HashMap::new();
        metrics.insert("startup_ms".to_string(), 3000.0); // Over budget

        harness.record(BenchmarkResult {
            benchmark_type: BenchmarkType::ColdStartup,
            hardware_tier: HardwareTier::Medium,
            platform: "windows".to_string(),
            version: "0.0.1".to_string(),
            timestamp: now_secs(),
            metrics,
            within_budget: true, // Will be overwritten
        });

        assert!(!harness.results()[0].within_budget);
    }

    #[test]
    fn baseline_comparison() {
        let mut harness = BenchmarkHarness::new();
        
        let mut baseline_metrics = HashMap::new();
        baseline_metrics.insert("startup_ms".to_string(), 2000.0);
        
        harness.set_baseline(BenchmarkResult {
            benchmark_type: BenchmarkType::ColdStartup,
            hardware_tier: HardwareTier::Medium,
            platform: "windows".to_string(),
            version: "0.0.1".to_string(),
            timestamp: now_secs(),
            metrics: baseline_metrics,
            within_budget: true,
        });

        let mut current_metrics = HashMap::new();
        current_metrics.insert("startup_ms".to_string(), 1500.0);
        
        let current = BenchmarkResult {
            benchmark_type: BenchmarkType::ColdStartup,
            hardware_tier: HardwareTier::Medium,
            platform: "windows".to_string(),
            version: "0.0.2".to_string(),
            timestamp: now_secs(),
            metrics: current_metrics,
            within_budget: true,
        };

        let comparison = harness.compare_to_baseline(&current).unwrap();
        assert!(comparison.changes.get("startup_ms").unwrap() < &0.0); // Improved
    }

    #[test]
    fn generate_report() {
        let mut harness = BenchmarkHarness::new();
        let mut metrics = HashMap::new();
        metrics.insert("startup_ms".to_string(), 1500.0);

        harness.record(BenchmarkResult {
            benchmark_type: BenchmarkType::ColdStartup,
            hardware_tier: HardwareTier::Medium,
            platform: "windows".to_string(),
            version: "0.0.1".to_string(),
            timestamp: now_secs(),
            metrics,
            within_budget: true,
        });

        let report = harness.generate_report();
        assert!(report.contains("ColdStartup"));
        assert!(report.contains("1500"));
    }
}
