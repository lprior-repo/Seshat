import subprocess
import sys

binary = "target/debug/seshat"

def test(name, args, expect_exit=None, expect_stdout=None, expect_stderr=None):
    print(f"Testing: {name}")
    try:
        # pass args directly, some might be weird
        result = subprocess.run([binary] + args, capture_output=True, text=True)
        if expect_exit is not None and result.returncode != expect_exit:
            print(f"  FAILED: expected exit {expect_exit}, got {result.returncode}")
            print(f"  stdout: {result.stdout}")
            print(f"  stderr: {result.stderr}")
            return False
        return True
    except Exception as e:
        print(f"  CRASHED: {e}")
        return False

# Adversarial test cases
test("No arguments", [], expect_exit=2)
test("Only valid-command", ["valid-command"], expect_exit=0)
test("Simulate failure", ["simulate-failure"], expect_exit=1)
test("Complex state normal", ["complex-state", "--depth", "10"], expect_exit=0)
test("Complex state boundary max", ["complex-state", "--depth", "254"], expect_exit=0)
test("Complex state overflow", ["complex-state", "--depth", "255"], expect_exit=1)
test("Complex state negative", ["complex-state", "--depth", "-1"], expect_exit=1)
test("Complex state very negative", ["complex-state", "--depth", "-2147483648"], expect_exit=1)
test("Complex state underflow", ["complex-state", "--depth", "-2147483649"], expect_exit=2)
test("Complex state text", ["complex-state", "--depth", "abc"], expect_exit=2)
test("Empty argument", [""], expect_exit=2)
test("Null byte argument", ["\0"], expect_exit=2)
test("Lots of arguments", ["--help"] * 65536, expect_exit=2)
