use std::path::PathBuf;

use crate::pipeline::{Pipeline, Step};

/// Renders and executes pipeline steps, showing commands and output.
pub struct Presenter {
    step_number: usize,
    total_steps: usize,
    it_binary: PathBuf,
    delay_ms: u64,
}

impl Presenter {
    pub fn new(total_steps: usize, it_binary: PathBuf, delay_ms: u64) -> Self {
        Self {
            step_number: 0,
            total_steps,
            it_binary,
            delay_ms,
        }
    }

    pub fn print_header(&self, pipeline: &Pipeline) {
        let width = 72;
        println!("{}", "=".repeat(width));
        println!("  Pipeline: {}", pipeline.name);
        println!("  {}", pipeline.summary);
        println!("  Steps: {}", self.total_steps);
        println!("{}", "=".repeat(width));
        println!();
    }

    pub async fn execute_step(
        &mut self,
        step: &Step,
        server_addr: &str,
    ) -> anyhow::Result<()> {
        self.step_number += 1;

        // Separator
        println!("{}", "-".repeat(72));
        println!(
            "[{}/{}] {}",
            self.step_number, self.total_steps, step.description
        );
        println!();

        // Print the command
        let args_display = step.args.join(" ");
        println!("  $ it --server {} {}", server_addr, args_display);
        println!();

        // Execute
        let output = tokio::process::Command::new(&self.it_binary)
            .arg("--server")
            .arg(server_addr)
            .args(&step.args)
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Print stdout with indent
        if !stdout.is_empty() {
            for line in stdout.lines() {
                println!("  {}", line);
            }
            println!();
        }

        // Handle failures
        if !output.status.success() {
            if step.expect_failure {
                if !stderr.is_empty() {
                    for line in stderr.lines() {
                        println!("  [expected error] {}", line);
                    }
                }
                println!("  ^ This error was expected (demonstrating error handling)");
                println!();
            } else {
                if !stderr.is_empty() {
                    eprintln!("  STDERR:");
                    for line in stderr.lines() {
                        eprintln!("  {}", line);
                    }
                }
                anyhow::bail!(
                    "Step {}/{} failed with exit code {:?}: it {}",
                    self.step_number,
                    self.total_steps,
                    output.status.code(),
                    args_display,
                );
            }
        }

        // Verify output assertions
        if !step.expect_failure {
            for expected in &step.assert_stdout_contains {
                if !stdout.contains(expected.as_str()) {
                    anyhow::bail!(
                        "Step {}/{} output verification failed.\n  Expected stdout to contain: \"{}\"\n  Actual stdout:\n{}",
                        self.step_number,
                        self.total_steps,
                        expected,
                        stdout,
                    );
                }
            }
        }

        // Optional delay between steps
        if self.delay_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(self.delay_ms)).await;
        }

        Ok(())
    }

    pub fn print_footer(&self) {
        println!("{}", "=".repeat(72));
        println!(
            "  Pipeline complete. {}/{} steps executed successfully.",
            self.step_number, self.total_steps
        );
        println!("{}", "=".repeat(72));
    }
}
