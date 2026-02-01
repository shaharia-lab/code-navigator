use anyhow::Result;
use clap::Parser;
use code_navigator::benchmark::{BenchmarkMetrics, BenchmarkTimer};
use code_navigator::core::{CodeGraph, NodeType};
use code_navigator::parser::{GoParser, Language, PythonParser, TypeScriptParser};
use code_navigator::serializer::{csv, dot, fast_compressed, graphml, json, jsonl};
use colored::Colorize;

mod cli;
use cli::{Cli, Commands};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Load graph from file, auto-detecting format from extension
/// Phase 3 optimization: Try to load cached indices first
fn load_graph(path: &Path) -> Result<CodeGraph> {
    use code_navigator::serializer::index_cache::SerializedIndices;

    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("bin");

    // Load the graph data - use optimized binary format with JSON fallback
    let mut graph = match extension {
        "json" => json::load_from_file(path)?, // Legacy JSON support
        "jsonl" => jsonl::load_from_jsonl(&path.to_string_lossy())?, // Legacy JSONL support
        _ => fast_compressed::load_from_file(&path.to_string_lossy())?, // Default: optimized binary (with JSON fallback)
    };

    // Phase 3: Try to load cached indices
    let idx_path = path.with_extension("idx");
    if idx_path.exists() {
        if let Ok(cached_indices) = SerializedIndices::load(path) {
            let graph_hash = graph.compute_hash();
            // Validate cache matches current graph
            if cached_indices.validate(graph.nodes.len(), graph.edges.len(), &graph_hash) {
                // Cache is valid - apply it
                graph.apply_indices(cached_indices);
                return Ok(graph);
            }
        }
    }

    // No cache or cache invalid - build indices and save cache
    graph.build_indexes();

    // Save cache for next time
    let indices = graph.extract_indices();
    let _ = indices.save(path); // Ignore errors

    Ok(graph)
}

/// Detect changed files using git
fn detect_changed_files_git(directory: &Path, file_extension: &str) -> Result<Vec<PathBuf>> {
    // Get files changed compared to HEAD (includes both staged and unstaged)
    let output = Command::new("git")
        .arg("-C")
        .arg(directory)
        .arg("diff")
        .arg("--name-only")
        .arg("HEAD")
        .output()?;

    if !output.status.success() {
        anyhow::bail!("Git command failed");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut changed_files = Vec::new();

    for line in stdout.lines() {
        let path = directory.join(line);
        if path.extension().and_then(|s| s.to_str()) == Some(file_extension) && path.exists() {
            changed_files.push(path);
        }
    }

    // Also check for untracked files
    let output = Command::new("git")
        .arg("-C")
        .arg(directory)
        .arg("ls-files")
        .arg("--others")
        .arg("--exclude-standard")
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let path = directory.join(line);
            if path.extension().and_then(|s| s.to_str()) == Some(file_extension)
                && path.exists()
                && !changed_files.contains(&path)
            {
                changed_files.push(path);
            }
        }
    }

    Ok(changed_files)
}

/// Get current git commit hash
fn get_git_commit_hash(directory: &Path) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(directory)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Count lines of code in a file
fn count_lines_of_code(path: &Path) -> Result<usize> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let count = reader.lines().count();
    Ok(count)
}

/// Count total lines of code in all files with given extension
fn count_total_loc(directory: &Path, file_ext: &str) -> Result<usize> {
    use walkdir::WalkDir;

    let mut total = 0;
    for entry in WalkDir::new(directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some(file_ext))
    {
        if let Ok(loc) = count_lines_of_code(entry.path()) {
            total += loc;
        }
    }
    Ok(total)
}

/// Detect changed files using timestamps (fallback when git is not available)
fn detect_changed_files_timestamp(
    directory: &Path,
    existing_graph: &CodeGraph,
    file_extension: &str,
) -> Result<Vec<PathBuf>> {
    use std::fs;
    use walkdir::WalkDir;

    let mut changed_files = Vec::new();

    for entry in WalkDir::new(directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some(file_extension))
    {
        let path = entry.path();
        let path_str = path.to_string_lossy().to_string();

        // Check if file is new or modified
        if let Some(file_meta) = existing_graph.metadata.file_metadata.get(&path_str) {
            // File exists in graph, check if modified
            if let Ok(metadata) = fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    let modified_str = format!("{:?}", modified);
                    if modified_str != file_meta.last_modified {
                        changed_files.push(path.to_path_buf());
                    }
                }
            }
        } else {
            // New file
            changed_files.push(path.to_path_buf());
        }
    }

    Ok(changed_files)
}

/// Detect deleted files by comparing stored file metadata with current directory
fn detect_deleted_files(directory: &Path, existing_graph: &CodeGraph) -> Vec<String> {
    let mut deleted_files = Vec::new();

    for file_path in existing_graph.metadata.file_metadata.keys() {
        let full_path = if Path::new(file_path).is_absolute() {
            PathBuf::from(file_path)
        } else {
            directory.join(file_path)
        };

        if !full_path.exists() {
            deleted_files.push(file_path.clone());
        }
    }

    deleted_files
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Index {
            directory,
            output,
            language,
            exclude: _,
            include_tests: _,
            incremental,
            force,
            benchmark,
            benchmark_json,
        } => {
            let lang = language.as_deref().unwrap_or("go");

            // Determine file extension for the language
            let file_ext = match lang {
                "go" => "go",
                "typescript" | "ts" => "ts",
                "javascript" | "js" => "js",
                "python" | "py" => "py",
                _ => anyhow::bail!("Unsupported language: {}", lang),
            };

            // Initialize benchmark timer if requested
            let mut bench_timer = if *benchmark {
                Some(BenchmarkTimer::new())
            } else {
                None
            };

            // Count LOC if benchmarking
            let total_loc = if *benchmark {
                if !cli.quiet {
                    println!("{}", "Counting lines of code...".dimmed());
                }
                let discovery_start = std::time::Instant::now();
                let loc = count_total_loc(directory, file_ext)?;
                if let Some(ref mut timer) = bench_timer {
                    timer.discovery_duration = Some(discovery_start.elapsed());
                }
                loc
            } else {
                0
            };

            // Check if incremental mode is requested
            let should_use_incremental = *incremental && !force && output.exists();

            let graph = if should_use_incremental {
                // INCREMENTAL MODE
                if !cli.quiet {
                    println!("{}", "Incremental update mode...".green().bold());
                }

                // Load existing graph
                let mut existing_graph = match load_graph(output) {
                    Ok(g) => {
                        if !cli.quiet {
                            println!(
                                "{} Loaded existing graph ({} nodes)",
                                "✓".green().bold(),
                                g.nodes.len().to_string().cyan()
                            );
                        }
                        g
                    }
                    Err(e) => {
                        if !cli.quiet {
                            println!("{} Failed to load existing graph: {}", "⚠".yellow(), e);
                            println!("{} Performing full generation...", "→".blue());
                        }
                        CodeGraph::new(directory.to_string_lossy().to_string(), lang.to_string())
                    }
                };

                // Try git first, fallback to timestamps
                let (changed_files, detection_method) =
                    match detect_changed_files_git(directory, file_ext) {
                        Ok(files) => (files, "git"),
                        Err(_) => {
                            if !cli.quiet {
                                println!(
                                    "{} Git not available, using file timestamps",
                                    "⚠".yellow()
                                );
                            }
                            (
                                detect_changed_files_timestamp(
                                    directory,
                                    &existing_graph,
                                    file_ext,
                                )?,
                                "timestamps",
                            )
                        }
                    };

                // Detect deleted files
                let deleted_files = detect_deleted_files(directory, &existing_graph);

                if !cli.quiet {
                    println!(
                        "{} Detected {} changed files via {}",
                        "✓".green().bold(),
                        changed_files.len().to_string().cyan(),
                        detection_method
                    );
                    if !deleted_files.is_empty() {
                        println!(
                            "{} Detected {} deleted files",
                            "✓".green().bold(),
                            deleted_files.len().to_string().red()
                        );
                    }
                }

                // Remove deleted files
                for deleted_file in &deleted_files {
                    existing_graph.remove_nodes_from_file(deleted_file);
                }

                // Remove and reparse changed files
                let total_files_before = existing_graph.metadata.file_metadata.len();
                for changed_file in &changed_files {
                    let file_str = changed_file.to_string_lossy().to_string();
                    existing_graph.remove_nodes_from_file(&file_str);
                }

                // Parse changed files
                use std::fs;

                let files_to_parse: HashSet<_> = changed_files.iter().collect();
                let mut files_parsed = 0;

                // Create temporary parser based on language
                match lang {
                    "go" => {
                        let mut parser = GoParser::new()?;
                        for file_path in &files_to_parse {
                            if let Err(e) = parser.parse_file(file_path, &mut existing_graph) {
                                if !cli.quiet {
                                    println!(
                                        "{} Failed to parse {}: {}",
                                        "⚠".yellow(),
                                        file_path.display(),
                                        e
                                    );
                                }
                            } else {
                                files_parsed += 1;
                                // Track file metadata
                                if let Ok(metadata) = fs::metadata(file_path) {
                                    if let Ok(modified) = metadata.modified() {
                                        existing_graph.track_file_metadata(
                                            file_path,
                                            format!("{:?}", modified),
                                        );
                                    }
                                }
                            }
                        }
                    }
                    "typescript" | "ts" => {
                        let mut parser = TypeScriptParser::new(Language::TypeScript)?;
                        for file_path in &files_to_parse {
                            if let Err(e) = parser.parse_file(file_path, &mut existing_graph) {
                                if !cli.quiet {
                                    println!(
                                        "{} Failed to parse {}: {}",
                                        "⚠".yellow(),
                                        file_path.display(),
                                        e
                                    );
                                }
                            } else {
                                files_parsed += 1;
                                if let Ok(metadata) = fs::metadata(file_path) {
                                    if let Ok(modified) = metadata.modified() {
                                        existing_graph.track_file_metadata(
                                            file_path,
                                            format!("{:?}", modified),
                                        );
                                    }
                                }
                            }
                        }
                    }
                    "javascript" | "js" => {
                        let mut parser = TypeScriptParser::new(Language::JavaScript)?;
                        for file_path in &files_to_parse {
                            if let Err(e) = parser.parse_file(file_path, &mut existing_graph) {
                                if !cli.quiet {
                                    println!(
                                        "{} Failed to parse {}: {}",
                                        "⚠".yellow(),
                                        file_path.display(),
                                        e
                                    );
                                }
                            } else {
                                files_parsed += 1;
                                if let Ok(metadata) = fs::metadata(file_path) {
                                    if let Ok(modified) = metadata.modified() {
                                        existing_graph.track_file_metadata(
                                            file_path,
                                            format!("{:?}", modified),
                                        );
                                    }
                                }
                            }
                        }
                    }
                    "python" | "py" => {
                        let mut parser = PythonParser::new()?;
                        for file_path in &files_to_parse {
                            if let Err(e) = parser.parse_file(file_path, &mut existing_graph) {
                                if !cli.quiet {
                                    println!(
                                        "{} Failed to parse {}: {}",
                                        "⚠".yellow(),
                                        file_path.display(),
                                        e
                                    );
                                }
                            } else {
                                files_parsed += 1;
                                if let Ok(metadata) = fs::metadata(file_path) {
                                    if let Ok(modified) = metadata.modified() {
                                        existing_graph.track_file_metadata(
                                            file_path,
                                            format!("{:?}", modified),
                                        );
                                    }
                                }
                            }
                        }
                    }
                    _ => unreachable!(),
                }

                // Update metadata
                existing_graph.metadata.generated_at = chrono::Utc::now().to_rfc3339();
                existing_graph.metadata.stats.files_parsed = files_parsed;
                existing_graph.metadata.stats.total_nodes = existing_graph.nodes.len();
                existing_graph.metadata.stats.total_edges = existing_graph.edges.len();
                existing_graph.metadata.git_commit_hash = get_git_commit_hash(directory);

                let files_cached =
                    total_files_before - deleted_files.len() - changed_files.len() + files_parsed;

                if !cli.quiet {
                    let node_change = existing_graph.nodes.len() as i32 - total_files_before as i32;
                    let sign = if node_change >= 0 { "+" } else { "" };
                    println!(
                        "{} Updated graph with {} nodes ({}{}) and {} edges",
                        "✓".green().bold(),
                        existing_graph.nodes.len().to_string().cyan(),
                        sign,
                        node_change.to_string().yellow(),
                        existing_graph.edges.len().to_string().cyan()
                    );
                    println!(
                        "  {} Files parsed: {}",
                        "→".blue(),
                        files_parsed.to_string().cyan()
                    );
                    println!(
                        "  {} Files cached: {}",
                        "→".blue(),
                        files_cached.to_string().green()
                    );
                }

                existing_graph
            } else {
                // FULL GENERATION MODE
                if !cli.quiet {
                    println!("{}", "Indexing codebase...".green().bold());
                }

                let mut new_graph =
                    CodeGraph::new(directory.to_string_lossy().to_string(), lang.to_string());

                // Start timing parse phase
                let parse_start = if bench_timer.is_some() {
                    Some(std::time::Instant::now())
                } else {
                    None
                };

                match lang {
                    "go" => {
                        let mut parser = GoParser::new()?;
                        parser.parse_directory(directory, &mut new_graph)?;
                    }
                    "typescript" | "ts" => {
                        let mut parser = TypeScriptParser::new(Language::TypeScript)?;
                        parser.parse_directory(directory, &mut new_graph)?;
                    }
                    "javascript" | "js" => {
                        let mut parser = TypeScriptParser::new(Language::JavaScript)?;
                        parser.parse_directory(directory, &mut new_graph)?;
                    }
                    "python" | "py" => {
                        let mut parser = PythonParser::new()?;
                        parser.parse_directory(directory, &mut new_graph)?;
                    }
                    _ => unreachable!(),
                }

                // Record parse duration
                if let (Some(ref mut timer), Some(start)) = (&mut bench_timer, parse_start) {
                    timer.parsing_duration = Some(start.elapsed());
                }

                // Track all files in metadata
                use std::fs;
                use walkdir::WalkDir;
                for entry in WalkDir::new(directory)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some(file_ext))
                {
                    let path = entry.path();
                    if let Ok(metadata) = fs::metadata(path) {
                        if let Ok(modified) = metadata.modified() {
                            new_graph.track_file_metadata(
                                &path.to_path_buf(),
                                format!("{:?}", modified),
                            );
                        }
                    }
                }

                new_graph.metadata.git_commit_hash = get_git_commit_hash(directory);

                if !cli.quiet {
                    println!(
                        "{} Indexed {} nodes and {} edges",
                        "✓".green().bold(),
                        new_graph.metadata.stats.total_nodes.to_string().cyan(),
                        new_graph.metadata.stats.total_edges.to_string().cyan()
                    );
                    println!(
                        "  {} Files parsed: {}",
                        "→".blue(),
                        new_graph.metadata.stats.files_parsed.to_string().cyan()
                    );
                }

                new_graph
            };

            // Save in binary format (compressed)
            let serialization_start = if bench_timer.is_some() {
                Some(std::time::Instant::now())
            } else {
                None
            };

            fast_compressed::save_to_file(&graph, &output.to_string_lossy())?;

            // Record serialization duration
            if let (Some(ref mut timer), Some(start)) = (&mut bench_timer, serialization_start) {
                timer.serialization_duration = Some(start.elapsed());
            }

            if !cli.quiet {
                println!(
                    "  {} Output: {}",
                    "→".blue(),
                    output.display().to_string().cyan()
                );
            }

            // Display benchmark results if enabled
            if *benchmark {
                if let Some(timer) = bench_timer {
                    use std::fs;
                    let output_size = fs::metadata(output).map(|m| m.len()).unwrap_or(0);

                    let metrics = BenchmarkMetrics::new(
                        &timer,
                        total_loc,
                        graph.metadata.stats.files_parsed,
                        lang.to_string(),
                        graph.nodes.len(),
                        graph.edges.len(),
                        output_size,
                    );

                    metrics.display();

                    // Export to JSON if requested
                    if let Some(json_path) = benchmark_json {
                        match metrics.to_json() {
                            Ok(json) => {
                                if let Err(e) = fs::write(json_path, json) {
                                    eprintln!(
                                        "{} Failed to write benchmark JSON: {}",
                                        "⚠".yellow(),
                                        e
                                    );
                                } else if !cli.quiet {
                                    println!(
                                        "  {} Benchmark JSON: {}",
                                        "→".blue(),
                                        json_path.display().to_string().cyan()
                                    );
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "{} Failed to serialize benchmark data: {}",
                                    "⚠".yellow(),
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }

        Commands::Query {
            graph: graph_file,
            output,
            count,
            limit,
            name,
            r#type,
            package,
            file,
            tag: _,
        } => {
            use std::time::Instant;

            let load_start = Instant::now();
            let graph = load_graph(graph_file)?;
            let load_time = load_start.elapsed();

            let query_start = Instant::now();

            // Phase 1 Optimization: Use index-based queries instead of linear scans
            // Apply filters in optimal order (most selective first)

            let mut nodes: Vec<&code_navigator::core::Node> = Vec::new();
            let mut using_index = false;

            // Priority 1: Exact name match (O(1) hash lookup)
            if let Some(name_filter) = name {
                if !name_filter.contains('*') {
                    // Exact match - use by_name index
                    nodes = graph.get_nodes_by_name(name_filter);
                    using_index = true;
                } else {
                    // Wildcard pattern - need to scan all nodes
                    if !using_index {
                        nodes = graph.nodes.iter().collect();
                        using_index = true;
                    }
                    let pattern = name_filter.replace('*', "");
                    nodes.retain(|n| n.name.contains(&pattern));
                }
            }

            // Priority 2: Type filter (O(1) hash lookup)
            if let Some(type_filter) = r#type {
                let node_type = match type_filter.as_str() {
                    "function" => NodeType::Function,
                    "method" => NodeType::Method,
                    "handler" => NodeType::HttpHandler,
                    "middleware" => NodeType::Middleware,
                    _ => anyhow::bail!("Unknown node type: {}", type_filter),
                };

                if !using_index {
                    // No previous filter - use type index directly
                    nodes = graph.get_nodes_by_type(&node_type);
                    using_index = true;
                } else {
                    // Intersect with existing results using O(k) where k = result size
                    let type_nodes = graph.get_nodes_by_type(&node_type);
                    let type_set: HashSet<_> = type_nodes.iter().map(|n| &n.id).collect();
                    nodes.retain(|n| type_set.contains(&n.id));
                }
            }

            // If no indexed filters applied yet, start with all nodes
            if !using_index {
                nodes = graph.nodes.iter().collect();
            }

            // Priority 3: Package filter (O(n) scan on filtered results)
            if let Some(package_filter) = package {
                nodes.retain(|n| n.package == *package_filter);
            }

            // Priority 4: File filter (O(n) scan on filtered results)
            if let Some(file_filter) = file {
                nodes.retain(|n| n.file_path.to_string_lossy().contains(file_filter));
            }

            if let Some(limit_count) = limit {
                nodes.truncate(*limit_count);
            }

            let query_time = query_start.elapsed();

            // Print timing info in verbose mode or as a comment
            if cli.verbose {
                eprintln!(
                    "⏱  Load time: {:.3}s | Query time: {:.3}s",
                    load_time.as_secs_f64(),
                    query_time.as_secs_f64()
                );
            }

            if *count {
                println!("{}", nodes.len());
                return Ok(());
            }

            match output.as_str() {
                "table" => {
                    if nodes.is_empty() {
                        println!("{}", "No nodes found".yellow());
                        return Ok(());
                    }

                    println!(
                        "{:<40} {:<15} {:<30} {:<10}",
                        "Name".bold(),
                        "Type".bold(),
                        "Package".bold(),
                        "Line".bold()
                    );
                    println!("{}", "-".repeat(95));

                    for node in &nodes {
                        let type_str = match node.node_type {
                            NodeType::Function => "Function".green(),
                            NodeType::Method => "Method".blue(),
                            NodeType::HttpHandler => "HTTP Handler".yellow(),
                            NodeType::Middleware => "Middleware".magenta(),
                        };

                        println!(
                            "{:<40} {:<15} {:<30} {:<10}",
                            node.name,
                            format!("{}", type_str),
                            node.package,
                            node.line
                        );
                    }

                    println!();
                    println!(
                        "{} {} nodes found",
                        "→".blue(),
                        nodes.len().to_string().cyan()
                    );
                }
                "json" => {
                    let json = serde_json::to_string_pretty(&nodes)?;
                    println!("{}", json);
                }
                "tree" => {
                    for node in &nodes {
                        println!("├─ {}", node.name.cyan().bold());
                        println!("│  └─ Type: {:?}", node.node_type);
                        println!("│  └─ Package: {}", node.package);
                        println!("│  └─ File: {}", node.file_path.display());
                        println!("│  └─ Line: {}", node.line);
                        println!();
                    }
                }
                _ => anyhow::bail!("Unknown output format: {}", output),
            }
        }

        Commands::Trace {
            graph: graph_file,
            from,
            depth,
            output,
            show_lines,
            filter: _,
        } => {
            let graph = load_graph(graph_file)?;

            // Find the starting node
            let nodes = graph.get_nodes_by_name(from);
            if nodes.is_empty() {
                anyhow::bail!("Function not found: {}", from);
            }

            let start_node = nodes[0];
            let traces = graph.trace_dependencies(&start_node.id, *depth);

            if traces.is_empty() {
                if !cli.quiet {
                    println!("{}", "No dependencies found".yellow());
                }
                return Ok(());
            }

            match output.as_str() {
                "tree" => {
                    println!("{}", format!("Dependencies of {}", from).bold());
                    println!();

                    let mut current_depth = 0;
                    for trace in &traces {
                        if trace.depth > current_depth {
                            current_depth = trace.depth;
                        }

                        let indent = "  ".repeat(trace.depth);
                        let line_info = if *show_lines {
                            format!(" ({}:{})", trace.file_path.display(), trace.line)
                        } else {
                            String::new()
                        };

                        println!(
                            "{}├─ {}{}",
                            indent,
                            trace.to_name.cyan(),
                            line_info.dimmed()
                        );
                    }

                    println!();
                    println!("{} {} dependencies found", "→".blue(), traces.len());
                }
                "json" => {
                    let json = serde_json::to_string_pretty(&traces)?;
                    println!("{}", json);
                }
                _ => anyhow::bail!("Unknown output format: {}", output),
            }
        }

        Commands::Callers {
            graph: graph_file,
            function,
            count,
            output,
            show_lines,
        } => {
            let graph = load_graph(graph_file)?;
            let callers = graph.find_callers(function);

            if *count {
                println!("{}", callers.len());
                return Ok(());
            }

            if callers.is_empty() {
                if !cli.quiet {
                    println!("{}", format!("No callers found for {}", function).yellow());
                }
                return Ok(());
            }

            match output.as_str() {
                "tree" => {
                    println!("{}", format!("Callers of {}", function).bold());
                    println!();

                    for caller in &callers {
                        let line_info = if *show_lines {
                            format!(" ({}:{})", caller.file_path.display(), caller.line)
                        } else {
                            String::new()
                        };

                        // Try to get the calling function name from the node
                        if let Some(node) = graph.get_node_by_id(&caller.from) {
                            println!("├─ {}{}", node.name.cyan(), line_info.dimmed());
                        } else {
                            println!("├─ {}{}", caller.from.cyan(), line_info.dimmed());
                        }
                    }

                    println!();
                    println!("{} {} callers found", "→".blue(), callers.len());
                }
                "json" => {
                    let json = serde_json::to_string_pretty(&callers)?;
                    println!("{}", json);
                }
                "table" => {
                    println!(
                        "{:<40} {:<30} {:<10}",
                        "Caller".bold(),
                        "File".bold(),
                        "Line".bold()
                    );
                    println!("{}", "-".repeat(80));

                    for caller in &callers {
                        let caller_name = graph
                            .get_node_by_id(&caller.from)
                            .map(|n| n.name.as_str())
                            .unwrap_or(&caller.from);

                        println!(
                            "{:<40} {:<30} {:<10}",
                            caller_name,
                            caller
                                .file_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or(""),
                            caller.line
                        );
                    }

                    println!();
                    println!("{} {} callers found", "→".blue(), callers.len());
                }
                _ => anyhow::bail!("Unknown output format: {}", output),
            }
        }

        Commands::Path {
            graph: graph_file,
            from,
            to,
            limit,
            all,
            max_depth,
            output,
        } => {
            let graph = load_graph(graph_file)?;

            // Find the starting node
            let from_nodes = graph.get_nodes_by_name(from);
            if from_nodes.is_empty() {
                anyhow::bail!("Starting function not found: {}", from);
            }

            let from_node = from_nodes[0];

            let paths = if let Some(n) = limit {
                // Find N paths using DFS with early termination
                let mut found_paths = graph.find_paths_limited(&from_node.id, to, *max_depth, *n);
                found_paths.sort_by_key(|p| p.len());
                found_paths
            } else if *all {
                // Find all paths (warning: may be very slow)
                let mut found_paths = graph.find_paths_limited(&from_node.id, to, *max_depth, usize::MAX);
                found_paths.sort_by_key(|p| p.len());
                found_paths
            } else {
                // Default: Find shortest path using BFS (fastest)
                if let Some(shortest_path) = graph.find_shortest_path(&from_node.id, to, *max_depth) {
                    vec![shortest_path]
                } else {
                    Vec::new()
                }
            };

            if paths.is_empty() {
                if !cli.quiet {
                    println!(
                        "{}",
                        format!("No path found from {} to {}", from, to).yellow()
                    );
                }
                return Ok(());
            }

            match output.as_str() {
                "tree" => {
                    println!("{}", format!("Paths from {} to {}", from, to).bold());
                    println!();

                    for (idx, path) in paths.iter().enumerate() {
                        println!("{} Path {} (length: {})", "→".blue(), idx + 1, path.len());
                        for (i, step) in path.iter().enumerate() {
                            let prefix = if i == path.len() - 1 {
                                "└─"
                            } else {
                                "├─"
                            };
                            println!("  {} {}", prefix, step.cyan());
                        }
                        println!();
                    }

                    println!("{} {} paths found", "→".blue(), paths.len());
                }
                "json" => {
                    let json = serde_json::to_string_pretty(&paths)?;
                    println!("{}", json);
                }
                _ => anyhow::bail!("Unknown output format: {}", output),
            }
        }

        Commands::Analyze {
            graph: graph_file,
            analysis_type,
            threshold,
            limit,
            output,
        } => {
            let graph = load_graph(graph_file)?;

            match analysis_type.as_str() {
                "complexity" => {
                    let mut results: Vec<_> = graph
                        .nodes
                        .iter()
                        .map(|node| {
                            let metrics = graph.get_complexity(&node.id);
                            (node, metrics)
                        })
                        .collect();

                    // Sort by combined complexity
                    results.sort_by(|a, b| {
                        (b.1.fan_in + b.1.fan_out).cmp(&(a.1.fan_in + a.1.fan_out))
                    });

                    if let Some(limit_count) = limit {
                        results.truncate(*limit_count);
                    }

                    match output.as_str() {
                        "table" => {
                            println!(
                                "{:<40} {:<10} {:<10} {:<10}",
                                "Function".bold(),
                                "Fan-In".bold(),
                                "Fan-Out".bold(),
                                "Cyclomatic".bold()
                            );
                            println!("{}", "-".repeat(70));

                            for (node, metrics) in &results {
                                println!(
                                    "{:<40} {:<10} {:<10} {:<10}",
                                    node.name, metrics.fan_in, metrics.fan_out, metrics.cyclomatic
                                );
                            }

                            println!();
                            println!("{} {} nodes analyzed", "→".blue(), results.len());
                        }
                        "json" => {
                            let json_results: Vec<_> = results
                                .iter()
                                .map(|(node, metrics)| {
                                    serde_json::json!({
                                        "name": node.name,
                                        "fan_in": metrics.fan_in,
                                        "fan_out": metrics.fan_out,
                                        "cyclomatic": metrics.cyclomatic
                                    })
                                })
                                .collect();
                            let json = serde_json::to_string_pretty(&json_results)?;
                            println!("{}", json);
                        }
                        _ => anyhow::bail!("Unknown output format: {}", output),
                    }
                }

                "hotspots" => {
                    let limit_count = limit.unwrap_or(20);
                    let hotspots = graph.find_hotspots(limit_count);

                    if hotspots.is_empty() {
                        println!("{}", "No hotspots found".yellow());
                        return Ok(());
                    }

                    match output.as_str() {
                        "table" => {
                            println!("{:<50} {:<15}", "Function".bold(), "Call Count".bold());
                            println!("{}", "-".repeat(65));

                            for hotspot in &hotspots {
                                println!("{:<50} {:<15}", hotspot.name, hotspot.call_count);
                            }

                            println!();
                            println!("{} {} hotspots found", "→".blue(), hotspots.len());
                        }
                        "json" => {
                            let json = serde_json::to_string_pretty(&hotspots)?;
                            println!("{}", json);
                        }
                        _ => anyhow::bail!("Unknown output format: {}", output),
                    }
                }

                "coupling" => {
                    let threshold_val = threshold.unwrap_or(5);
                    let mut coupling_data: std::collections::HashMap<String, usize> =
                        std::collections::HashMap::new();

                    for edge in &graph.edges {
                        // Extract package from node ID or edge
                        if let Some(from_node) = graph.get_node_by_id(&edge.from) {
                            let package = from_node.package.clone();
                            *coupling_data.entry(package).or_insert(0) += 1;
                        }
                    }

                    let mut results: Vec<_> = coupling_data
                        .into_iter()
                        .filter(|(_, count)| *count >= threshold_val)
                        .collect();

                    results.sort_by(|a, b| b.1.cmp(&a.1));

                    if let Some(limit_count) = limit {
                        results.truncate(*limit_count);
                    }

                    println!("{:<40} {:<15}", "Package".bold(), "Dependencies".bold());
                    println!("{}", "-".repeat(55));

                    for (package, count) in &results {
                        println!("{:<40} {:<15}", package, count);
                    }

                    println!();
                    println!("{} {} packages above threshold", "→".blue(), results.len());
                }

                "circular" => {
                    println!(
                        "{}",
                        "Circular dependency detection not yet implemented".yellow()
                    );
                    println!("Coming soon!");
                }

                _ => anyhow::bail!(
                    "Unknown analysis type: {}. Use: complexity, hotspots, coupling, circular",
                    analysis_type
                ),
            }
        }

        Commands::Export {
            graph: graph_file,
            output,
            format,
            filter,
            exclude_tests,
        } => {
            let mut graph = load_graph(graph_file)?;

            // Apply filters if specified
            if filter.is_some() || *exclude_tests {
                let mut package_filter = None;
                let mut type_filter = None;

                if let Some(filter_str) = filter {
                    let parts: Vec<&str> = filter_str.split(':').collect();
                    if parts.len() == 2 {
                        match parts[0] {
                            "package" => package_filter = Some(parts[1]),
                            "type" => {
                                type_filter = match parts[1] {
                                    "function" => Some(NodeType::Function),
                                    "method" => Some(NodeType::Method),
                                    "handler" => Some(NodeType::HttpHandler),
                                    "middleware" => Some(NodeType::Middleware),
                                    _ => anyhow::bail!("Unknown node type: {}", parts[1]),
                                };
                            }
                            _ => anyhow::bail!(
                                "Unknown filter type: {}. Use: package:NAME or type:TYPE",
                                parts[0]
                            ),
                        }
                    } else {
                        anyhow::bail!("Invalid filter format. Use: package:NAME or type:TYPE");
                    }
                }

                graph = graph.filter(package_filter, type_filter.as_ref(), *exclude_tests);

                if !cli.quiet && (filter.is_some() || *exclude_tests) {
                    println!(
                        "{} Filtered to {} nodes and {} edges",
                        "→".blue(),
                        graph.nodes.len().to_string().cyan(),
                        graph.edges.len().to_string().cyan()
                    );
                }
            }

            if !cli.quiet {
                println!(
                    "{}",
                    format!("Exporting to {} format...", format).green().bold()
                );
            }

            match format.as_str() {
                "graphml" => {
                    graphml::save_to_file(&graph, output)?;
                    if !cli.quiet {
                        println!(
                            "{} Exported to GraphML: {}",
                            "✓".green().bold(),
                            output.display()
                        );
                    }
                }
                "dot" => {
                    dot::save_to_file(&graph, output)?;
                    if !cli.quiet {
                        println!(
                            "{} Exported to DOT: {}",
                            "✓".green().bold(),
                            output.display()
                        );
                    }
                }
                "csv" => {
                    csv::save_to_files(&graph, output)?;
                    if !cli.quiet {
                        println!("{} Exported to CSV files", "✓".green().bold());
                    }
                }
                _ => anyhow::bail!("Unknown export format: {}. Use: graphml, dot, csv", format),
            }
        }

        Commands::Extract {
            graph: graph_file,
            from,
            depth,
            output,
        } => {
            let graph = load_graph(graph_file)?;

            if !cli.quiet {
                println!(
                    "{}",
                    format!(
                        "Extracting subgraph from '{}' with depth {}...",
                        from, depth
                    )
                    .green()
                    .bold()
                );
            }

            // Extract subgraph
            let subgraph = graph.extract_subgraph(from, *depth);

            if subgraph.nodes.is_empty() {
                anyhow::bail!("No nodes found starting from '{}'", from);
            }

            // Save in binary format (compressed)
            fast_compressed::save_to_file(&subgraph, &output.to_string_lossy())?;

            if !cli.quiet {
                println!(
                    "{} Extracted subgraph with {} nodes and {} edges",
                    "✓".green().bold(),
                    subgraph.nodes.len().to_string().cyan(),
                    subgraph.edges.len().to_string().cyan()
                );
                println!(
                    "  {} Output: {}",
                    "→".blue(),
                    output.display().to_string().cyan()
                );
            }
        }

        Commands::Diff {
            old_graph,
            new_graph,
            show_added,
            show_removed,
            show_changed,
            complexity_threshold,
            output,
        } => {
            let old = load_graph(old_graph)?;
            let new = load_graph(new_graph)?;

            if !cli.quiet {
                println!("{}", "Comparing graphs...".green().bold());
            }

            let diff = old.diff(&new);

            match output.as_str() {
                "json" => {
                    let json = serde_json::to_string_pretty(&diff)?;
                    println!("{}", json);
                }
                "table" => {
                    // Summary
                    println!("\n{}", "=== GRAPH DIFF SUMMARY ===".bold());
                    println!(
                        "Added nodes:   {}",
                        diff.added_nodes.len().to_string().green()
                    );
                    println!(
                        "Removed nodes: {}",
                        diff.removed_nodes.len().to_string().red()
                    );
                    println!(
                        "Changed nodes: {}",
                        diff.changed_nodes.len().to_string().yellow()
                    );
                    println!(
                        "Edge changes:  {} added, {} removed",
                        diff.added_edges_count.to_string().green(),
                        diff.removed_edges_count.to_string().red()
                    );

                    // Show added nodes if requested or if no specific flags
                    if (*show_added || (!show_added && !show_removed && !show_changed))
                        && !diff.added_nodes.is_empty()
                    {
                        println!("\n{}", "=== ADDED NODES ===".green().bold());
                        for node_id in &diff.added_nodes {
                            println!("  {} {}", "+".green(), node_id);
                        }
                    }

                    // Show removed nodes
                    if (*show_removed || (!show_added && !show_removed && !show_changed))
                        && !diff.removed_nodes.is_empty()
                    {
                        println!("\n{}", "=== REMOVED NODES ===".red().bold());
                        for node_id in &diff.removed_nodes {
                            println!("  {} {}", "-".red(), node_id);
                        }
                    }

                    // Show changed nodes
                    if (*show_changed || (!show_added && !show_removed && !show_changed))
                        && !diff.changed_nodes.is_empty()
                    {
                        println!("\n{}", "=== CHANGED NODES ===".yellow().bold());
                        for change in &diff.changed_nodes {
                            println!(
                                "  {} {} (line {} → {})",
                                "~".yellow(),
                                change.node_name,
                                change.old_line,
                                change.new_line
                            );
                            if change.old_signature != change.new_signature {
                                println!("    Old: {}", change.old_signature.dimmed());
                                println!("    New: {}", change.new_signature);
                            }
                        }
                    }

                    // Show complexity changes if threshold specified
                    if let Some(threshold) = complexity_threshold {
                        let significant_changes: Vec<_> = diff
                            .complexity_changes
                            .iter()
                            .filter(|c| c.change.abs() >= *threshold as i32)
                            .collect();

                        if !significant_changes.is_empty() {
                            println!(
                                "\n{}",
                                format!("=== COMPLEXITY CHANGES (≥{}) ===", threshold).bold()
                            );
                            for change in significant_changes {
                                let arrow = if change.change > 0 {
                                    "↑".red()
                                } else {
                                    "↓".green()
                                };
                                println!("  {} {} ({:+})", arrow, change.node_name, change.change);
                                println!(
                                    "    Fan-in:  {} → {}",
                                    change.old_fan_in, change.new_fan_in
                                );
                                println!(
                                    "    Fan-out: {} → {}",
                                    change.old_fan_out, change.new_fan_out
                                );
                            }
                        }
                    }

                    println!();
                }
                _ => anyhow::bail!("Unknown output format: {}. Use: table, json", output),
            }
        }
    }

    Ok(())
}
