use super::dev_team::CodePatch;
use crate::tools::build_tools::BuildTools;
use std::sync::Mutex;

pub struct QASwarm {
    last_error: Mutex<String>,
}

impl QASwarm {
    pub fn new() -> Self {
        Self {
            last_error: Mutex::new(String::new()),
        }
    }

    pub async fn test_patch(&self, _patch: &CodePatch, project_path: Option<&str>) -> bool {
        let path = project_path.unwrap_or("../baton-gateway-engine");
        eprintln!("[QA-SWARM] Running cargo check on {}...", path);
        
        match BuildTools::cargo_check(path) {
            Ok(_) => {
                eprintln!("[QA-SWARM] ✅ QA Verification PASSED.");
                true
            }
            Err(e) => {
                eprintln!("[QA-SWARM] ❌ QA Verification FAILED.");
                *self.last_error.lock().unwrap() = e;
                false
            }
        }
    }

    pub fn get_feedback(&self) -> String {
        self.last_error.lock().unwrap().clone()
    }
}
