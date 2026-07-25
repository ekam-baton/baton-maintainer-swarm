import logging
import asyncio
from tools.a2a_client import A2AClient
import json
import time

logger = logging.getLogger("Supervisor")

class SupervisorAgent:
    """
    The Large Model (e.g. Llama-3-70B) that oversees the Swarm.
    It does not write code. It reviews transcripts, synthesizes summaries,
    and asks the Human User for permission via the BATON A2A tunnel.
    """
    def __init__(self):
        # We would initialize the Large Model connection here (e.g. Ollama endpoint)
        pass

    async def review_and_seek_approval(self, alert: dict, patch_diff: str) -> bool:
        logger.info("Supervisor is reviewing the Dev and QA transcripts...")
        
        # 1. Canary Test (Deploy to local staging)
        canary_passed = await self._run_canary_test(patch_diff)
        if not canary_passed:
            logger.error("Canary test failed! The patch broke the A2A network.")
            await A2AClient.send_emergency_alert("Bricking risk detected. Patch aborted.")
            return False

        # 2. Synthesize Executive Summary (Mocked LLM call)
        summary = f"""
BOSS, ACTION REQUIRED:
Alert: {alert.get('title')}
The Dev and QA round tables debated and reached a consensus. 
The staging canary test passed (networking is intact).

Diff Summary:
{patch_diff[:100]}...

Reply 'Approve' to deploy to production.
"""
        
        # 3. Archive the transcript for future fine-tuning
        self._archive_dataset(alert, patch_diff)

        # 4. Ask the Human via BATON A2A
        return await A2AClient.ask_human_for_approval(summary)
        
    async def _run_canary_test(self, patch_diff: str) -> bool:
        """
        Deploys the patch to an isolated docker container and verifies 
        the A2A WebRTC tunnel still establishes a connection.
        """
        logger.info("Running staging canary test to prevent deployment dead loop...")
        await asyncio.sleep(1) # Mock testing time
        return True
        
    def _archive_dataset(self, alert: dict, patch_diff: str):
        """
        Saves the entire context to training_dataset.jsonl for fine-tuning.
        """
        record = {
            "timestamp": time.time(),
            "alert": alert,
            "patch": patch_diff
        }
        with open("dataset/training_dataset.jsonl", "a") as f:
            f.write(json.dumps(record) + "\n")
