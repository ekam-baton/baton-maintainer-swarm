use std::io::{self, Write};
use tokio::time::{timeout, Duration};

pub struct SupervisorAgent;

impl SupervisorAgent {
    pub fn new() -> Self {
        Self
    }

    pub async fn review_and_seek_approval(&self, title: &str, diff: &str) -> bool {
        eprintln!("[SUPERVISOR] Reviewing final patch for '{}'...", title);
        eprintln!("[SUPERVISOR] Diff summary:\n{}", diff);
        eprint!("[SUPERVISOR] Approve this patch? (yes/no): ");
        let _ = io::stdout().flush();
        
        let prompt_task = tokio::task::spawn_blocking(|| {
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap_or(0);
            input.trim().eq_ignore_ascii_case("yes")
        });

        match timeout(Duration::from_secs(30 * 60), prompt_task).await {
            Ok(Ok(approved)) => approved,
            _ => false,
        }
    }
}
