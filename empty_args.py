import os
import sys

child_pid = os.fork()
if child_pid == 0:
    os.execv("target/debug/seshat", [])
else:
    _, status = os.waitpid(child_pid, 0)
    exit_code = os.waitstatus_to_exitcode(status)
    print(f"Empty argv exit code: {exit_code}")
