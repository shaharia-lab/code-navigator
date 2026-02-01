use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Comprehensive benchmark metrics for indexing operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetrics {
    /// Total lines of code processed
    pub total_loc: usize,
    /// Total files processed
    pub total_files: usize,
    /// Language being indexed
    pub language: String,
    /// Total nodes in graph
    pub total_nodes: usize,
    /// Total edges in graph
    pub total_edges: usize,

    /// Timing breakdown in milliseconds
    pub timing_ms: TimingBreakdown,

    /// Memory metrics in megabytes
    pub memory_mb: MemoryMetrics,

    /// Output file metrics
    pub output: OutputMetrics,

    /// Efficiency ratios
    pub throughput: ThroughputMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingBreakdown {
    pub total: u64,
    pub discovery: u64,
    pub parsing: u64,
    pub merging: u64,
    pub index_build: u64,
    pub serialization: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub peak_mb: f64,
    pub graph_mb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputMetrics {
    pub file_size_mb: f64,
    pub compression_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputMetrics {
    pub files_per_sec: f64,
    pub loc_per_sec: f64,
    pub nodes_per_sec: f64,
    pub nodes_per_file: f64,
    pub edges_per_node: f64,
}

/// Helper to track timing phases
pub struct BenchmarkTimer {
    pub start_time: Instant,
    pub discovery_duration: Option<Duration>,
    pub parsing_duration: Option<Duration>,
    pub merging_duration: Option<Duration>,
    pub index_build_duration: Option<Duration>,
    pub serialization_duration: Option<Duration>,
}

impl BenchmarkTimer {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            discovery_duration: None,
            parsing_duration: None,
            merging_duration: None,
            index_build_duration: None,
            serialization_duration: None,
        }
    }

    pub fn total_elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl BenchmarkMetrics {
    /// Create metrics from collected data
    pub fn new(
        timer: &BenchmarkTimer,
        total_loc: usize,
        total_files: usize,
        language: String,
        total_nodes: usize,
        total_edges: usize,
        output_size_bytes: u64,
    ) -> Self {
        let total_ms = timer.total_elapsed().as_millis() as u64;
        let total_sec = total_ms as f64 / 1000.0;

        // Calculate timing breakdown
        let discovery_ms = timer.discovery_duration.map_or(0, |d| d.as_millis() as u64);
        let parsing_ms = timer.parsing_duration.map_or(0, |d| d.as_millis() as u64);
        let merging_ms = timer.merging_duration.map_or(0, |d| d.as_millis() as u64);
        let index_build_ms = timer.index_build_duration.map_or(0, |d| d.as_millis() as u64);
        let serialization_ms = timer.serialization_duration.map_or(0, |d| d.as_millis() as u64);

        // Calculate throughput
        let files_per_sec = if total_sec > 0.0 { total_files as f64 / total_sec } else { 0.0 };
        let loc_per_sec = if total_sec > 0.0 { total_loc as f64 / total_sec } else { 0.0 };
        let nodes_per_sec = if total_sec > 0.0 { total_nodes as f64 / total_sec } else { 0.0 };
        let nodes_per_file = if total_files > 0 { total_nodes as f64 / total_files as f64 } else { 0.0 };
        let edges_per_node = if total_nodes > 0 { total_edges as f64 / total_nodes as f64 } else { 0.0 };

        // Memory estimation (rough estimate based on data structures)
        let estimated_graph_mb = (total_nodes * 200 + total_edges * 100) as f64 / 1_048_576.0;

        // Output metrics
        let output_size_mb = output_size_bytes as f64 / 1_048_576.0;
        let uncompressed_estimate = (total_nodes * 200 + total_edges * 100) as f64 / 1_048_576.0;
        let compression_ratio = if uncompressed_estimate > 0.0 {
            output_size_mb / uncompressed_estimate
        } else {
            1.0
        };

        Self {
            total_loc,
            total_files,
            language,
            total_nodes,
            total_edges,
            timing_ms: TimingBreakdown {
                total: total_ms,
                discovery: discovery_ms,
                parsing: parsing_ms,
                merging: merging_ms,
                index_build: index_build_ms,
                serialization: serialization_ms,
            },
            memory_mb: MemoryMetrics {
                peak_mb: estimated_graph_mb * 1.5, // Rough estimate with overhead
                graph_mb: estimated_graph_mb,
            },
            output: OutputMetrics {
                file_size_mb: output_size_mb,
                compression_ratio,
            },
            throughput: ThroughputMetrics {
                files_per_sec,
                loc_per_sec,
                nodes_per_sec,
                nodes_per_file,
                edges_per_node,
            },
        }
    }

    /// Display formatted benchmark results
    pub fn display(&self) {
        println!("\n{}", "=== BENCHMARK RESULTS ===".bold().green());

        // Codebase stats
        println!(
            "{:<18} {} files | {} LOC",
            "Codebase:".bold(),
            format_number(self.total_files).cyan(),
            format_number(self.total_loc).cyan()
        );

        // Graph stats
        println!(
            "{:<18} {} nodes | {} edges ({:.2} edges/node)",
            "Graph:".bold(),
            format_number(self.total_nodes).cyan(),
            format_number(self.total_edges).cyan(),
            self.throughput.edges_per_node
        );

        // Timing breakdown
        let total_sec = self.timing_ms.total as f64 / 1000.0;
        println!(
            "{:<18} {:.1}s total",
            "Timing:".bold(),
            total_sec
        );

        // Always show discovery if it was measured (even if fast)
        if self.timing_ms.discovery > 0 || total_sec < 1.0 {
            let pct = percentage(self.timing_ms.discovery, self.timing_ms.total);
            let discovery_sec = self.timing_ms.discovery as f64 / 1000.0;
            if discovery_sec < 0.1 {
                println!(
                    "  {:<16} <0.1s ({:.1}%)",
                    "Discovery:".dimmed(),
                    pct
                );
            } else {
                println!(
                    "  {:<16} {:.1}s ({:.1}%)",
                    "Discovery:".dimmed(),
                    discovery_sec,
                    pct
                );
            }
        }

        if self.timing_ms.parsing > 0 {
            let pct = percentage(self.timing_ms.parsing, self.timing_ms.total);
            println!(
                "  {:<16} {:.1}s ({:.1}%)",
                "Parsing:".dimmed(),
                self.timing_ms.parsing as f64 / 1000.0,
                pct
            );
        }

        if self.timing_ms.merging > 0 {
            let pct = percentage(self.timing_ms.merging, self.timing_ms.total);
            println!(
                "  {:<16} {:.1}s ({:.1}%)",
                "Merging:".dimmed(),
                self.timing_ms.merging as f64 / 1000.0,
                pct
            );
        }

        if self.timing_ms.index_build > 0 {
            let pct = percentage(self.timing_ms.index_build, self.timing_ms.total);
            println!(
                "  {:<16} {:.1}s ({:.1}%)",
                "Indexing:".dimmed(),
                self.timing_ms.index_build as f64 / 1000.0,
                pct
            );
        }

        if self.timing_ms.serialization > 0 {
            let pct = percentage(self.timing_ms.serialization, self.timing_ms.total);
            println!(
                "  {:<16} {:.1}s ({:.1}%)",
                "Saving:".dimmed(),
                self.timing_ms.serialization as f64 / 1000.0,
                pct
            );
        }

        // Memory
        println!(
            "{:<18} {:.1} MB peak",
            "Memory:".bold(),
            self.memory_mb.peak_mb
        );

        // Output
        println!(
            "{:<18} {:.2} MB ({:.0}% compression)",
            "Output:".bold(),
            self.output.file_size_mb,
            (1.0 - self.output.compression_ratio) * 100.0
        );

        // Throughput
        println!(
            "{:<18} {} files/sec | {} LOC/sec",
            "Throughput:".bold(),
            format_number(self.throughput.files_per_sec as usize).green(),
            format_number(self.throughput.loc_per_sec as usize).green()
        );

        println!();
    }

    /// Export as JSON
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

fn format_number(n: usize) -> String {
    let s = n.to_string();
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();

    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }

    result
}

fn percentage(part: u64, total: u64) -> f64 {
    if total > 0 {
        (part as f64 / total as f64) * 100.0
    } else {
        0.0
    }
}
