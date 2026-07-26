#[path = "../src/supervisor.rs"]
mod supervisor;
#[path = "../src/teams/mod.rs"]
mod teams;
#[path = "../src/tools/mod.rs"]
mod tools;

use supervisor::SupervisorAgent;
use teams::dev_team::DevSwarm;
use teams::qa_team::QASwarm;
use tools::build_tools::BuildTools;
use tools::sandbox::SandboxEnv;

#[tokio::test]
async fn test_dev_and_qa_swarm_workflow() {
    let dev_swarm = DevSwarm::new();
    let patch = dev_swarm.debate_and_patch("CVE-2026-9999").await;

    assert!(patch.security_clearance);
    assert!(!patch.diff.is_empty());

    let qa_swarm = QASwarm::new();
    let qa_result = qa_swarm.test_patch(&patch).await;
    assert!(qa_result, "QA Swarm should approve valid constant-time patch");
}

#[tokio::test]
async fn test_supervisor_approval() {
    let supervisor = SupervisorAgent::new();
    let approved = supervisor.review_and_seek_approval("Test Alert", "diff --git a/main.rs").await;
    assert!(approved, "Supervisor should approve valid patch");
}

#[test]
fn test_canary_deployment_and_rollback() {
    let canary_res = BuildTools::canary_deploy(".");
    assert!(canary_res.is_ok(), "Canary deployment simulation should succeed");

    let rollback_res = BuildTools::auto_rollback(".");
    assert!(rollback_res.is_ok(), "Auto rollback simulation should execute");
}

#[test]
fn test_sandbox_env_instantiation() {
    let sandbox = SandboxEnv::new("rust:latest");
    // Sandbox struct instantiates cleanly
    let _ = sandbox;
}
