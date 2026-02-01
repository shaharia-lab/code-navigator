use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "code-navigator")]
#[command(about = "AI-First Source Code Navigation System", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Quiet mode (errors only)
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Index a codebase to build a navigable code graph
    Index {
        /// Directory to parse
        directory: PathBuf,

        /// Output file
        #[arg(short, long, default_value = "codenav.bin")]
        output: PathBuf,

        /// Language: go, typescript, python (auto-detect if not specified)
        #[arg(short, long)]
        language: Option<String>,

        /// Exclude files matching pattern (can be specified multiple times)
        #[arg(short, long)]
        exclude: Vec<String>,

        /// Include test files
        #[arg(long)]
        include_tests: bool,

        /// Enable incremental updates (parse only changed files)
        #[arg(long)]
        incremental: bool,

        /// Force full reindexing even with --incremental
        #[arg(long)]
        force: bool,
    },

    /// Query nodes in the graph
    Query {
        /// Graph file
        #[arg(short, long, default_value = "codenav.bin")]
        graph: PathBuf,

        /// Output format: table, json, tree
        #[arg(short, long, default_value = "table")]
        output: String,

        /// Show count only
        #[arg(short, long)]
        count: bool,

        /// Limit results
        #[arg(long)]
        limit: Option<usize>,

        /// Filter by name (supports wildcards)
        #[arg(long)]
        name: Option<String>,

        /// Filter by type: function, method, handler
        #[arg(long)]
        r#type: Option<String>,

        /// Filter by package
        #[arg(long)]
        package: Option<String>,

        /// Filter by file path
        #[arg(long)]
        file: Option<String>,

        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,
    },

    /// Trace function dependencies (what does this call?)
    Trace {
        /// Graph file
        #[arg(short, long, default_value = "codenav.bin")]
        graph: PathBuf,

        /// Function or method name to trace from
        #[arg(long)]
        from: String,

        /// Traversal depth (default: 1)
        #[arg(short, long, default_value = "1")]
        depth: usize,

        /// Output format: tree, json, dot
        #[arg(short, long, default_value = "tree")]
        output: String,

        /// Show line numbers
        #[arg(long)]
        show_lines: bool,

        /// Filter by pattern
        #[arg(short, long)]
        filter: Option<String>,
    },

    /// Find what calls a function (reverse dependencies)
    Callers {
        /// Graph file
        #[arg(short, long, default_value = "codenav.bin")]
        graph: PathBuf,

        /// Function or method name
        function: String,

        /// Show count only
        #[arg(short, long)]
        count: bool,

        /// Output format: tree, json, table
        #[arg(short, long, default_value = "tree")]
        output: String,

        /// Show line numbers
        #[arg(long)]
        show_lines: bool,
    },

    /// Find call paths between two functions
    Path {
        /// Graph file
        #[arg(short, long, default_value = "codenav.bin")]
        graph: PathBuf,

        /// Starting function
        #[arg(long)]
        from: String,

        /// Target function
        #[arg(long)]
        to: String,

        /// Show only shortest path
        #[arg(long)]
        shortest: bool,

        /// Show all paths (default: first 10)
        #[arg(long)]
        all: bool,

        /// Maximum search depth
        #[arg(long, default_value = "10")]
        max_depth: usize,

        /// Output format: tree, json
        #[arg(short, long, default_value = "tree")]
        output: String,
    },

    /// Analyze graph for metrics and insights
    Analyze {
        /// Graph file
        #[arg(short, long, default_value = "codenav.bin")]
        graph: PathBuf,

        /// Analysis type: complexity, coupling, hotspots, circular
        analysis_type: String,

        /// Threshold for reporting
        #[arg(long)]
        threshold: Option<usize>,

        /// Limit results
        #[arg(long)]
        limit: Option<usize>,

        /// Output format: table, json
        #[arg(short, long, default_value = "table")]
        output: String,
    },

    /// Export graph in different formats
    Export {
        /// Graph file
        #[arg(short, long, default_value = "codenav.bin")]
        graph: PathBuf,

        /// Output file
        #[arg(short, long)]
        output: PathBuf,

        /// Format: graphml, dot, csv
        #[arg(short, long)]
        format: String,

        /// Filter nodes (format: package:NAME or type:TYPE)
        #[arg(long)]
        filter: Option<String>,

        /// Exclude test files
        #[arg(long)]
        exclude_tests: bool,
    },

    /// Extract focused subgraph rooted at a node
    Extract {
        /// Graph file
        #[arg(short, long, default_value = "codenav.bin")]
        graph: PathBuf,

        /// Starting node (function/method name)
        #[arg(long)]
        from: String,

        /// Traversal depth (default: 2)
        #[arg(short, long, default_value = "2")]
        depth: usize,

        /// Output file
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Compare two graphs to detect changes
    Diff {
        /// Old graph file (baseline)
        old_graph: PathBuf,

        /// New graph file (current)
        new_graph: PathBuf,

        /// Show added nodes
        #[arg(long)]
        show_added: bool,

        /// Show removed nodes
        #[arg(long)]
        show_removed: bool,

        /// Show changed nodes
        #[arg(long)]
        show_changed: bool,

        /// Warn if complexity increases by this threshold
        #[arg(long)]
        complexity_threshold: Option<usize>,

        /// Output format: table, json
        #[arg(short, long, default_value = "table")]
        output: String,
    },
}
