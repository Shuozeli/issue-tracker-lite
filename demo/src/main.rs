use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

mod pipeline;
mod presenter;
mod server_harness;

#[derive(Parser)]
#[command(name = "it-demo", about = "Issue Tracker Demo Runner")]
struct DemoCli {
    /// Pipeline name to run
    pipeline: Option<String>,

    /// List available pipelines
    #[arg(long)]
    list: bool,

    /// Delay in milliseconds between steps (for live demos)
    #[arg(long, default_value = "200")]
    delay_ms: u64,

    /// Path to the `it` CLI binary
    #[arg(long, env = "IT_BINARY_PATH")]
    it_binary: Option<String>,
}

fn find_it_binary(explicit: Option<&str>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        let p = PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
        anyhow::bail!("specified it binary not found: {}", path);
    }

    // Auto-detect from workspace
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let candidates = [
        workspace_root.join("target/debug/it"),
        workspace_root.join("target/release/it"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return Ok(candidate.clone());
        }
    }

    anyhow::bail!(
        "could not find `it` binary. Run `cargo build -p issuetracker-cli` first. \
         Or specify --it-binary <path>."
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = DemoCli::parse();
    let pipelines = pipeline::all_pipelines();

    if cli.list {
        println!("Available pipelines:");
        println!();
        for p in &pipelines {
            println!("  {:<20} {}", p.name, p.summary);
        }
        println!();
        println!("Usage: it-demo <pipeline-name>");
        return Ok(());
    }

    let pipeline_name = match &cli.pipeline {
        Some(name) => name,
        None => {
            eprintln!("Error: no pipeline specified. Use --list to see available pipelines.");
            std::process::exit(1);
        }
    };

    let pipeline = pipelines
        .into_iter()
        .find(|p| p.name == pipeline_name.as_str())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "unknown pipeline: '{}'. Use --list to see available pipelines.",
                pipeline_name
            )
        })?;

    let it_binary = find_it_binary(cli.it_binary.as_deref())?;

    println!("Starting demo server...");
    let server = server_harness::DemoServer::start().await?;
    println!("Server running on {}", server.server_addr());
    println!();

    let mut presenter = presenter::Presenter::new(pipeline.steps.len(), it_binary, cli.delay_ms);

    presenter.print_header(&pipeline);

    for step in &pipeline.steps {
        presenter.execute_step(step, &server.server_addr()).await?;
    }

    presenter.print_footer();

    // Server drops here, triggering clean shutdown
    drop(server);

    Ok(())
}
