import asyncio
import hmac
import hashlib
import json
import logging
import os
from typing import Dict, Any

from fastapi import FastAPI, Request, HTTPException, Header, BackgroundTasks
from pydantic import BaseModel, Field
import uvicorn

from supervisor import SupervisorAgent
from teams.dev_team import DevSwarm
from teams.qa_team import QASwarm
from tools.a2a_client import A2AClient

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("MaintainerSwarm")

WEBHOOK_SECRET = os.getenv("SWARM_WEBHOOK_SECRET", "baton-super-secret-key-2026")

app = FastAPI(title="BATON Maintainer Swarm Listener", version="1.0.0")

class AlertPayload(BaseModel):
    title: str = Field(..., description="Short summary of the alert or CVE")
    description: str = Field(..., description="Detailed log content or CVE vulnerability metadata")
    timestamp: int = Field(..., description="Epoch timestamp when alert was generated")

def verify_signature(payload_bytes: bytes, signature: str) -> bool:
    """Verifies HMAC-SHA256 signature from the Rust Gateway."""
    if not signature:
        return False
    expected = hmac.new(
        WEBHOOK_SECRET.encode("utf-8"),
        payload_bytes,
        hashlib.sha256
    ).hexdigest()
    return hmac.compare_digest(expected, signature)

async def handle_alert(alert: AlertPayload):
    """
    Main entry point for handling an incoming alert (e.g. from the Gateway Engine).
    """
    alert_payload = alert.model_dump()
    logger.info(f"Processing alert: {alert_payload.get('title')}")
    
    # 1. Spin up the Dev Swarm to propose a patch
    dev_swarm = DevSwarm()
    proposed_patch = await dev_swarm.debate_and_patch(alert_payload)
    
    # 2. Hand off to QA Swarm for brutal testing
    qa_swarm = QASwarm()
    qa_passed = await qa_swarm.test_patch(proposed_patch)
    
    # Retry loop (max 5 iterations)
    iteration = 1
    while not qa_passed and iteration < 5:
        logger.warning(f"QA rejected patch. Dev team retrying. Iteration {iteration}")
        feedback = qa_swarm.get_feedback()
        proposed_patch = await dev_swarm.revise_patch(proposed_patch, feedback)
        qa_passed = await qa_swarm.test_patch(proposed_patch)
        iteration += 1
        
    if not qa_passed:
        logger.error("Swarm failed to reach consensus. Halting and alerting human.")
        await A2AClient.send_emergency_alert("Swarm failed to patch CVE. Human intervention required.")
        return
        
    # 3. Handoff to Supervisor Large Model
    supervisor = SupervisorAgent()
    approved = await supervisor.review_and_seek_approval(alert_payload, proposed_patch.diff)
    
    if approved:
        logger.info("Human approved the patch! Deploying...")
    else:
        logger.info("Human rejected the patch.")

@app.post("/webhook/alert")
async def webhook_alert(
    request: Request,
    background_tasks: BackgroundTasks,
    x_baton_signature: str = Header(None, alias="X-Baton-Signature")
):
    """
    Hardened Webhook Endpoint for Rust Gateway Alerts.
    Requires valid HMAC-SHA256 signature.
    """
    body_bytes = await request.body()
    
    # Security Layer 1: HMAC Verification
    if not verify_signature(body_bytes, x_baton_signature):
        logger.warning("Rejected unauthenticated webhook request! Invalid/missing HMAC signature.")
        raise HTTPException(status_code=401, detail="Invalid cryptographic signature")
        
    try:
        data = json.loads(body_bytes)
        alert = AlertPayload(**data)
    except Exception as e:
        logger.warning(f"Invalid payload format: {e}")
        raise HTTPException(status_code=400, detail="Malformed payload")

    # Queue processing in background so Rust gateway receives instant 200 OK
    background_tasks.add_task(handle_alert, alert)
    return {"status": "accepted", "message": "Alert queued for Swarm analysis"}

if __name__ == "__main__":
    # Security Layer 2: Bind strictly to 127.0.0.1 (Loopback isolation)
    uvicorn.run(app, host="127.0.0.1", port=8000)
