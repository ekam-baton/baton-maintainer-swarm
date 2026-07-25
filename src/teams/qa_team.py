import logging
import random

logger = logging.getLogger("QATeam")

class QASwarm:
    """
    A multi-agent swarm consisting of a Penetration Tester, Unit Tester, and Performance Tester.
    Their sole job is to break the Dev Team's patch.
    """
    def __init__(self):
        self.last_feedback = ""

    async def test_patch(self, patch: str) -> bool:
        logger.info("QA Team Round Table: testing the dev team's patch...")
        
        # Simulate the adversarial debate
        logger.info("Pen Tester: 'I'm fuzzing the new endpoint...'")
        logger.info("Unit Tester: 'Running cargo test...'")
        
        # 20% chance they find a flaw to simulate the retry loop
        if random.random() < 0.2:
            self.last_feedback = "The constant-time comparison fails when the payload is null."
            logger.warning(f"QA Team REJECTED the patch: {self.last_feedback}")
            return False
            
        logger.info("QA Team APPROVED the patch. No exploits found.")
        return True
        
    def get_feedback(self) -> str:
        return self.last_feedback
