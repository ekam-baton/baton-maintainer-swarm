import asyncio
import json
import logging
from typing import Dict, Any

from supervisor import SupervisorAgent
from teams.dev_team import DevSwarm
from teams.qa_team import QASwarm
from tools.a2a_client import A2AClient

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("MaintainerSwarm")

async def handle_alert(alert_payload: Dict[str, Any]):
    """
    Main entry point for handling an incoming alert (e.g. from the Gateway Engine).
    """
    logger.info(f"Received alert: {alert_payload.get('title')}")
    
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
        # TODO: call git_tools and build_tools to merge and deploy
    else:
        logger.info("Human rejected the patch.")

if __name__ == "__main__":
    # Mock payload for testing
    mock_cve = {
        "title": "CVE-2026-9999",
        "description": "Critical flaw in the cryptographic signature validation."
    }
    asyncio.run(handle_alert(mock_cve))
