//! RPC Endpoint Evaluator Binary
//!
//! Probes RPC endpoints to evaluate their capabilities and generates reports.
//!
//! Usage:
//!   cargo run --features evaluator --bin rpc-evaluator -- --chain-id 1
//!   cargo run --features evaluator --bin rpc-evaluator -- --chain-id 0 --format json -o report.json

use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

use web3_rpc_pool::endpoint::{EndpointCapabilities, EndpointGrade};
use web3_rpc_pool::presets;

#[derive(Parser, Debug)]
#[command(name = "rpc-evaluator", about = "Evaluate RPC endpoint capabilities")]
struct Args {
    /// Chain ID to evaluate (0 = all chains)
    #[arg(short, long)]
    chain_id: u64,

    /// Output format: table or json
    #[arg(short, long, default_value = "table")]
    format: String,

    /// Output file path (stdout if not specified)
    #[arg(short, long)]
    output: Option<String>,

    /// Max concurrent evaluations
    #[arg(long, default_value = "4")]
    concurrency: usize,

    /// Request timeout in seconds
    #[arg(long, default_value = "10")]
    timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EndpointReport {
    name: String,
    url: String,
    chain_id: u64,
    chain_name: String,
    reachable: bool,
    avg_latency_ms: Option<u64>,
    capabilities: EndpointCapabilities,
    grade: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct EvaluationReport {
    timestamp: String,
    endpoints: Vec<EndpointReport>,
    summary: ReportSummary,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReportSummary {
    total: usize,
    reachable: usize,
    unreachable: usize,
    grade_a: usize,
    grade_b: usize,
    grade_c: usize,
    grade_d: usize,
    grade_f: usize,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    result: Option<serde_json::Value>,
    error: Option<serde_json::Value>,
    #[allow(dead_code)]
    id: serde_json::Value,
}

async fn rpc_call(
    client: &Client,
    url: &str,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });

    let resp = client
        .post(url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("HTTP {}", status));
    }

    let json: JsonRpcResponse = resp
        .json()
        .await
        .map_err(|e| format!("JSON parse error: {}", e))?;

    if let Some(err) = json.error {
        return Err(format!("RPC error: {}", err));
    }

    json.result.ok_or_else(|| "No result in response".to_string())
}

async fn rpc_batch_call(
    client: &Client,
    url: &str,
    batch_size: usize,
) -> Result<(), String> {
    let batch: Vec<serde_json::Value> = (0..batch_size)
        .map(|i| {
            serde_json::json!({
                "jsonrpc": "2.0",
                "method": "eth_blockNumber",
                "params": [],
                "id": i + 1
            })
        })
        .collect();

    let resp = client
        .post(url)
        .json(&batch)
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("HTTP {}", status));
    }

    let results: Vec<JsonRpcResponse> = resp
        .json()
        .await
        .map_err(|e| format!("Batch parse error: {}", e))?;

    if results.len() != batch_size {
        return Err(format!(
            "Expected {} results, got {}",
            batch_size,
            results.len()
        ));
    }

    // Check that none have errors
    for r in &results {
        if r.error.is_some() {
            return Err("Batch response contains errors".to_string());
        }
    }

    Ok(())
}

async fn evaluate_endpoint(
    client: &Client,
    name: &str,
    url: &str,
    chain_id: u64,
) -> EndpointReport {
    let chain_name = presets::chain_name(chain_id).to_string();

    // Step 1: Connectivity test - eth_blockNumber x3
    let mut latencies = Vec::new();
    let mut reachable = false;
    let mut latest_block: Option<u64> = None;

    for _ in 0..3 {
        let start = Instant::now();
        match rpc_call(client, url, "eth_blockNumber", serde_json::json!([])).await {
            Ok(result) => {
                reachable = true;
                let elapsed = start.elapsed().as_millis() as u64;
                latencies.push(elapsed);
                if latest_block.is_none() {
                    if let Some(hex) = result.as_str() {
                        let hex = hex.trim_start_matches("0x");
                        latest_block = u64::from_str_radix(hex, 16).ok();
                    }
                }
            }
            Err(_) => {}
        }
    }

    if !reachable {
        return EndpointReport {
            name: name.to_string(),
            url: url.to_string(),
            chain_id,
            chain_name,
            reachable: false,
            avg_latency_ms: None,
            capabilities: EndpointCapabilities::default(),
            grade: EndpointGrade::F.to_string(),
        };
    }

    let avg_latency = if latencies.is_empty() {
        None
    } else {
        Some(latencies.iter().sum::<u64>() / latencies.len() as u64)
    };

    // Step 2: eth_getLogs test (10 block range)
    let supports_logs = if let Some(block) = latest_block {
        let from = if block > 10 { block - 10 } else { 0 };
        let params = serde_json::json!([{
            "fromBlock": format!("0x{:x}", from),
            "toBlock": format!("0x{:x}", block),
        }]);
        rpc_call(client, url, "eth_getLogs", params).await.is_ok()
    } else {
        false
    };

    // Step 3: Batch size test
    let batch_sizes = [1, 10, 50, 100, 500, 1000];
    let mut max_batch = 0u32;
    for &size in &batch_sizes {
        match rpc_batch_call(client, url, size).await {
            Ok(()) => {
                max_batch = size as u32;
            }
            Err(_) => break,
        }
    }
    // If all succeeded including 1000, treat as unlimited
    let max_batch_size = if max_batch >= 1000 { Some(0) } else { Some(max_batch) };

    // Step 4: Block range test for eth_getLogs
    let max_block_range = if supports_logs {
        if let Some(block) = latest_block {
            let ranges: [u64; 6] = [100, 1_000, 5_000, 10_000, 50_000, 100_000];
            let mut max_range = 0u64;
            for &range in &ranges {
                let from = if block > range { block - range } else { 0 };
                let params = serde_json::json!([{
                    "fromBlock": format!("0x{:x}", from),
                    "toBlock": format!("0x{:x}", block),
                }]);
                match rpc_call(client, url, "eth_getLogs", params).await {
                    Ok(_) => {
                        max_range = range;
                    }
                    Err(_) => break,
                }
            }
            // If all succeeded including 100K, treat as unlimited
            if max_range >= 100_000 {
                Some(0)
            } else {
                Some(max_range)
            }
        } else {
            Some(0)
        }
    } else {
        Some(0)
    };

    let capabilities = EndpointCapabilities {
        supports_eth_get_logs: Some(supports_logs),
        max_batch_size,
        max_block_range,
        supports_debug_trace: None,
        supports_websocket: false,
        rate_limit_rps: None,
    };

    let grade = capabilities.grade();

    EndpointReport {
        name: name.to_string(),
        url: url.to_string(),
        chain_id,
        chain_name,
        reachable,
        avg_latency_ms: avg_latency,
        capabilities,
        grade: grade.to_string(),
    }
}

fn print_table(report: &EvaluationReport) {
    println!(
        "\n{:<25} {:<6} {:<8} {:<10} {:<8} {:<10} {:<12}",
        "Name", "Grade", "Reach", "Latency", "Logs", "Batch", "BlockRange"
    );
    println!("{}", "-".repeat(85));

    let mut current_chain = 0u64;
    for ep in &report.endpoints {
        if ep.chain_id != current_chain {
            current_chain = ep.chain_id;
            println!(
                "\n--- {} (chain_id: {}) ---",
                ep.chain_name, ep.chain_id
            );
        }

        let latency = ep
            .avg_latency_ms
            .map(|l| format!("{}ms", l))
            .unwrap_or_else(|| "-".to_string());
        let logs = ep
            .capabilities
            .supports_eth_get_logs
            .map(|v| if v { "yes" } else { "no" })
            .unwrap_or("?");
        let batch = ep
            .capabilities
            .max_batch_size
            .map(|v| {
                if v == 0 {
                    "unlimited".to_string()
                } else {
                    v.to_string()
                }
            })
            .unwrap_or_else(|| "?".to_string());
        let range = ep
            .capabilities
            .max_block_range
            .map(|v| {
                if v == 0 {
                    "unlimited".to_string()
                } else {
                    format!("{}", v)
                }
            })
            .unwrap_or_else(|| "?".to_string());

        let reach = if ep.reachable { "OK" } else { "FAIL" };

        println!(
            "{:<25} {:<6} {:<8} {:<10} {:<8} {:<10} {:<12}",
            &ep.name[..ep.name.len().min(24)],
            ep.grade,
            reach,
            latency,
            logs,
            batch,
            range
        );
    }

    println!("\n--- Summary ---");
    println!("Total endpoints: {}", report.summary.total);
    println!(
        "Reachable: {} / Unreachable: {}",
        report.summary.reachable, report.summary.unreachable
    );
    println!(
        "Grades: A={} B={} C={} D={} F={}",
        report.summary.grade_a,
        report.summary.grade_b,
        report.summary.grade_c,
        report.summary.grade_d,
        report.summary.grade_f
    );
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let client = Client::builder()
        .timeout(Duration::from_secs(args.timeout))
        .build()
        .expect("Failed to build HTTP client");

    // Determine which chains to evaluate
    let chain_ids = if args.chain_id == 0 {
        presets::all_chain_ids()
    } else {
        vec![args.chain_id]
    };

    // Collect all endpoints
    let mut endpoints_to_eval: Vec<(String, String, u64)> = Vec::new();
    for &cid in &chain_ids {
        let endpoints = presets::default_endpoints(cid);
        if endpoints.is_empty() {
            eprintln!(
                "Warning: No endpoints found for chain {} ({})",
                presets::chain_name(cid),
                cid
            );
            continue;
        }
        for ep in endpoints {
            endpoints_to_eval.push((ep.name.clone(), ep.url.clone(), ep.chain_id));
        }
    }

    if endpoints_to_eval.is_empty() {
        eprintln!("No endpoints to evaluate. Check chain ID.");
        std::process::exit(1);
    }

    eprintln!(
        "Evaluating {} endpoints across {} chain(s) with concurrency={}...",
        endpoints_to_eval.len(),
        chain_ids.len(),
        args.concurrency
    );

    // Evaluate with semaphore-limited concurrency
    let semaphore = std::sync::Arc::new(Semaphore::new(args.concurrency));
    let client = std::sync::Arc::new(client);

    let mut handles = Vec::new();
    for (name, url, cid) in endpoints_to_eval {
        let sem = semaphore.clone();
        let client = client.clone();
        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            eprintln!("  Evaluating: {} ({})", name, url);
            evaluate_endpoint(&client, &name, &url, cid).await
        });
        handles.push(handle);
    }

    let mut reports: Vec<EndpointReport> = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(report) => reports.push(report),
            Err(e) => eprintln!("Task error: {}", e),
        }
    }

    // Sort by chain_id then name for stable output
    reports.sort_by(|a, b| a.chain_id.cmp(&b.chain_id).then(a.name.cmp(&b.name)));

    // Build summary
    let summary = ReportSummary {
        total: reports.len(),
        reachable: reports.iter().filter(|r| r.reachable).count(),
        unreachable: reports.iter().filter(|r| !r.reachable).count(),
        grade_a: reports.iter().filter(|r| r.grade == "A").count(),
        grade_b: reports.iter().filter(|r| r.grade == "B").count(),
        grade_c: reports.iter().filter(|r| r.grade == "C").count(),
        grade_d: reports.iter().filter(|r| r.grade == "D").count(),
        grade_f: reports.iter().filter(|r| r.grade == "F").count(),
    };

    let eval_report = EvaluationReport {
        timestamp: chrono::Utc::now().to_rfc3339(),
        endpoints: reports,
        summary,
    };

    // Output
    let output_str = match args.format.as_str() {
        "json" => serde_json::to_string_pretty(&eval_report).unwrap(),
        _ => {
            print_table(&eval_report);
            return;
        }
    };

    if let Some(path) = &args.output {
        std::fs::write(path, &output_str).expect("Failed to write output file");
        eprintln!("Report written to: {}", path);
    } else {
        println!("{}", output_str);
    }
}
