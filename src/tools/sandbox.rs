use std::process::Command;

pub struct SandboxEnv {
    image: String,
}

impl SandboxEnv {
    pub fn new(image: &str) -> Self {
        Self {
            image: image.to_string(),
        }
    }

    /// Executes code inside an ephemeral Docker container.
    /// Air-gapped: Network disabled, memory capped at 1GB.
    pub fn execute_command(&self, cmd: &str) -> Result<String, String> {
        eprintln!("[SANDBOX] Spinning up ephemeral container ({})", self.image);
        let output = Command::new("docker")
            .args([
                "run",
                "--rm",
                "--network", "none",
                "--memory", "1g",
                &self.image,
                "sh", "-c", cmd
            ])
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                if out.status.success() {
                    Ok(stdout)
                } else {
                    Err(format!("Sandbox command failed:\nSTDOUT: {}\nSTDERR: {}", stdout, stderr))
                }
            }
            Err(e) => Err(format!("Docker execution error: {}", e)),
        }
    }
}
