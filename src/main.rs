use axum::{
    http::{HeaderMap, StatusCode},
    routing::post,
    Json, Router,
};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::net::SocketAddr;

mod supervisor;
mod teams;
mod tools;

use supervisor::SupervisorAgent;
use teams::dev_team::DevSwarm;
use teams::qa_team::QASwarm;
use tools::build_tools::BuildTools;

use std::sync::OnceLock;

type HmacSha256 = Hmac<Sha256>;

static WEBHOOK_SECRET: OnceLock<String> = OnceLock::new();

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AlertPayload {
    pub title: String,
    pub description: String,
    pub timestamp: u64,
}

fn verify_hmac(payload_bytes: &[u8], signature_hex: &str) -> bool {
    let secret = WEBHOOK_SECRET.get().expect("WEBHOOK_SECRET not initialized");
    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(payload_bytes);
    let Ok(sig_bytes) = hex::decode(signature_hex) else {
        return false;
    };
    mac.verify_slice(&sig_bytes).is_ok()
}

async fn handle_alert(alert: AlertPayload) {
    eprintln!("[MAINTAINER-SWARM] 🚨 Handling Alert: {}", alert.title);

    let dev_swarm = DevSwarm::new();
    let mut patch = dev_swarm.debate_and_patch(&alert.title).await;

    let qa_swarm = QASwarm::new();
    let mut passed = qa_swarm.test_patch(&patch, None).await;

    let mut iteration = 1;
    while !passed && iteration < 5 {
        eprintln!("[MAINTAINER-SWARM] QA rejected patch. Iteration {}", iteration);
        let feedback = qa_swarm.get_feedback();
        dev_swarm.revise_patch(&mut patch, &feedback).await;
        passed = qa_swarm.test_patch(&patch, None).await;
        iteration += 1;
    }

    if !passed {
        eprintln!("[MAINTAINER-SWARM] ❌ Swarm failed to reach consensus. Halting.");
        return;
    }

    let supervisor = SupervisorAgent::new();
    let approved = supervisor.review_and_seek_approval(&alert.title, &patch.diff).await;

    if approved {
        eprintln!("[MAINTAINER-SWARM] ✅ Human Approved! Triggering Canary Deployment...");
        if let Err(e) = BuildTools::canary_deploy("../baton") {
            eprintln!("[MAINTAINER-SWARM] Canary deploy failed: {}. Triggering Auto-Rollback!", e);
            let _ = BuildTools::auto_rollback("../baton");
        } else {
            eprintln!("[MAINTAINER-SWARM] 🎉 Canary deployment successful!");
        }
    } else {
        eprintln!("[MAINTAINER-SWARM] 🛑 Human Rejected patch.");
    }
}

async fn webhook_handler(headers: HeaderMap, body_str: String) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let sig_header = headers
        .get("X-Baton-Signature")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if !verify_hmac(body_str.as_bytes(), sig_header) {
        eprintln!("[SECURITY] Rejected unauthenticated webhook request!");
        return Err((StatusCode::UNAUTHORIZED, "Invalid HMAC signature".to_string()));
    }

    let Ok(alert) = serde_json::from_str::<AlertPayload>(&body_str) else {
        return Err((StatusCode::BAD_REQUEST, "Malformed JSON payload".to_string()));
    };

    // Spawn background worker task so response returns immediately to Gateway
    tokio::spawn(async move {
        handle_alert(alert).await;
    });

    Ok(Json(serde_json::json!({
        "status": "accepted",
        "message": "Alert queued for Pure Rust Maintainer Swarm analysis"
    })))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let secret = std::env::var("WEBHOOK_SECRET").unwrap_or_else(|_| {
        eprintln!("[SECURITY] FATAL: WEBHOOK_SECRET environment variable is not set!");
        std::process::exit(1);
    });
    WEBHOOK_SECRET.set(secret).unwrap();

    let app = Router::new().route("/webhook/alert", post(webhook_handler));

    // Bind strictly to loopback 127.0.0.1 for zero-trust air-gapping
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    eprintln!("==================================================");
    eprintln!("BATON PURE RUST MAINTAINER SWARM 🦀");
    eprintln!("Listening on http://{}", addr);
    eprintln!("==================================================");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
