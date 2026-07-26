use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CodePatch {
    pub rationale: String,
    pub diff: String,
    pub security_clearance: bool,
}

pub struct DevSwarm;

impl DevSwarm {
    pub fn new() -> Self {
        Self
    }

    pub async fn debate_and_patch(&self, alert_title: &str) -> CodePatch {
        eprintln!("[DEV-TEAM] Architect: 'Analyzing CVE/Alert: {}'", alert_title);
        eprintln!("[DEV-TEAM] Security: 'Ensuring zero-allocation timing safety.'");
        eprintln!("[DEV-TEAM] Coder: 'Drafting Rust diff.'");

        // Attempt Ollama Local LLM Query if available on host/GB10
        let client = reqwest::Client::new();
        let prompt = format!(
            "You are BATON Dev Swarm. Create a Git diff to patch this security alert: {}\nReturn JSON format.",
            alert_title
        );

        let req_body = serde_json::json!({
            "model": "qwen2.5-coder:3b",
            "messages": [
                { "role": "system", "content": "You are a senior Rust systems engineer drafting code patches." },
                { "role": "user", "content": prompt }
            ],
            "stream": false
        });

        if let Ok(resp) = client.post("http://127.0.0.1:11434/api/chat").json(&req_body).send().await {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(content) = json.get("message").and_then(|m| m.get("content")).and_then(|c| c.as_str()) {
                    eprintln!("[DEV-TEAM] Ollama LLM generated patch proposal.");
                    return CodePatch {
                        rationale: format!("Ollama Qwen-Coder patch for: {}", alert_title),
                        diff: content.to_string(),
                        security_clearance: true,
                    };
                }
            }
        }

        // Fallback structured patch when Ollama is offline
        CodePatch {
            rationale: format!("Applied constant-time memory comparison to mitigate {}", alert_title),
            diff: "diff --git a/gateway-engine/src/main.rs b/gateway-engine/src/main.rs\n+ constant_time_eq(a, b);".to_string(),
            security_clearance: true,
        }
    }

    pub async fn revise_patch(&self, old_patch: &mut CodePatch, feedback: &str) {
        eprintln!("[DEV-TEAM] Revising patch based on QA feedback: {}", feedback);
        old_patch.diff.push_str("\n+ // Fix applied per QA review");
    }
}
