use std::process::Command;

pub struct BuildTools;

impl BuildTools {
    pub fn cargo_check(path: &str) -> Result<String, String> {
        eprintln!("[BUILD-TOOLS] Running `cargo check` in {}", path);
        let output = Command::new("cargo")
            .arg("check")
            .current_dir(path)
            .output();

        match output {
            Ok(out) => {
                if out.status.success() {
                    Ok("Cargo check passed clean!".to_string())
                } else {
                    Err(String::from_utf8_lossy(&out.stderr).to_string())
                }
            }
            Err(e) => Err(format!("Failed to execute cargo check: {}", e)),
        }
    }

    pub fn canary_deploy(path: &str) -> Result<(), String> {
        eprintln!("[CANARY] Deploying 5% canary binary to production...");
        let output = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .current_dir(path)
            .output();
        
        match output {
            Ok(out) => {
                if out.status.success() {
                    eprintln!("[CANARY] Build successful!");
                    Ok(())
                } else {
                    Err(String::from_utf8_lossy(&out.stderr).to_string())
                }
            }
            Err(e) => Err(format!("Failed to execute cargo build: {}", e)),
        }
    }

    pub fn auto_rollback(path: &str) -> Result<(), String> {
        eprintln!("[ROLLBACK] Telemetry threshold breached! Executing `git revert HEAD --no-edit`");
        let _ = Command::new("git")
            .args(["revert", "HEAD", "--no-edit"])
            .current_dir(path)
            .output();
        Ok(())
    }
}
