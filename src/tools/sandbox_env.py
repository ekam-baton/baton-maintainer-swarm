import docker
import logging

logger = logging.getLogger("DockerSandbox")

class SandboxEnv:
    """
    OpenHands / SWE-agent style Docker Sandbox.
    Prevents the LLM from executing malicious code on the host GB10 rig.
    All code compilation and test execution runs in an ephemeral container.
    """
    def __init__(self, image="rust:latest"):
        self.client = docker.from_env()
        self.image = image
        
    def execute_code(self, code: str, command: str) -> str:
        """
        Spins up a throwaway container, mounts the code, runs the command, 
        returns the output, and destroys the container instantly.
        """
        logger.info(f"Spinning up ephemeral {self.image} sandbox container...")
        try:
            # SWE-agent paradigm: Run in sandbox with no network access to host
            container = self.client.containers.run(
                self.image,
                command=f"sh -c '{command}'",
                remove=True,
                network_disabled=True,
                mem_limit="1g",
                detach=False
            )
            return container.decode('utf-8')
        except Exception as e:
            logger.error(f"Sandbox execution failed or crashed: {e}")
            return str(e)
