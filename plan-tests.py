import subprocess
import json
import os

P = "/home/lewis/.claude/skills/planner/planner.nu"
session_id = "seshat-tests6"


def run_nu(args):
    cmd = ["nu", P] + args
    print(f"Running: {' '.join(cmd[:3])} ...")
    res = subprocess.run(cmd, text=True, capture_output=True)
    if res.returncode != 0:
        print(f"Error: {res.stderr}")
    else:
        print(res.stdout)


# Initialize session
run_nu(
    [
        "init",
        "--session-id",
        session_id,
        "--description",
        "Implement all 240 test cases across 13 categories",
    ]
)


def add_task(id, title, desc):
    task = {
        "id": id,
        "title": title,
        "type": "task",
        "priority": 1,
        "effort": "2hr",
        "description": desc,
        "ears": {
            "ubiquitous": [
                "THE SYSTEM SHALL execute the specified test cases reliably"
            ],
            "event_driven": [
                {
                    "trigger": "WHEN tests are run",
                    "shall": "THE SYSTEM SHALL report pass/fail accurately",
                }
            ],
            "unwanted": [
                {
                    "condition": "IF a test fails",
                    "shall_not": "THE SYSTEM SHALL NOT crash the test runner",
                    "because": "other tests must run",
                }
            ],
        },
        "contracts": {
            "preconditions": ["Test harness is available"],
            "postconditions": ["All specified tests pass"],
            "invariants": ["Tests are deterministic"],
        },
        "tests": {
            "happy": [
                "All happy path scenarios in the category pass",
                "A secondary happy path passes",
            ],
            "error": [
                "All error path scenarios in the category fail gracefully",
                "Another error path scenario passes",
            ],
        },
    }
    task_json = json.dumps(task)
    run_nu(["add-task", "--task-json", task_json, session_id])


add_task(
    "task-001",
    "tests: Implement DOC-001 to DOC-020 (Document Invariants)",
    "Implement 20 test cases for Document and Scene Graph Invariants.",
)
add_task(
    "task-002",
    "tests: Implement GEO-001 to GEO-015 (Geometry Math)",
    "Implement first 15 test cases for Geometry & Transform Math.",
)
add_task(
    "task-003",
    "tests: Implement GEO-016 to GEO-030 (Geometry Math)",
    "Implement remaining 15 test cases for Geometry & Transform Math.",
)
add_task(
    "task-004",
    "tests: Implement SEL-001 to SEL-025 (Selection)",
    "Implement 25 test cases for Hit Testing & Selection.",
)
add_task(
    "task-005",
    "tests: Implement MUL-001 to MUL-018 (Multi-Selection)",
    "Implement first 18 test cases for Multi-Selection Transform.",
)
add_task(
    "task-006",
    "tests: Implement MUL-019 to MUL-037 (Multi-Selection)",
    "Implement remaining 19 test cases for Multi-Selection Transform.",
)
add_task(
    "task-007",
    "tests: Implement SUB-001 to SUB-017 (Subgraphs)",
    "Implement first 17 test cases for Subgraphs.",
)
add_task(
    "task-008",
    "tests: Implement SUB-018 to SUB-034 (Subgraphs)",
    "Implement remaining 17 test cases for Subgraphs.",
)
add_task(
    "task-009",
    "tests: Implement EDG-001 to EDG-017 (Edges)",
    "Implement first 17 test cases for Edges / Connectors.",
)
add_task(
    "task-010",
    "tests: Implement EDG-018 to EDG-035 (Edges)",
    "Implement remaining 18 test cases for Edges / Connectors.",
)
add_task(
    "task-011",
    "tests: Implement CAM-001 to CAM-012 (Viewport)",
    "Implement 12 test cases for Viewport, Zoom/Pan.",
)
add_task(
    "task-012",
    "tests: Implement SNP-001 to SNP-010 (Snapping)",
    "Implement 10 test cases for Snapping / Guides.",
)
add_task(
    "task-013",
    "tests: Implement CLP-001 to CLP-010 (Clipboard)",
    "Implement 10 test cases for Clipboard / Duplicate.",
)
add_task(
    "task-014",
    "tests: Implement HIS-001 to HIS-013 (History)",
    "Implement 13 test cases for Undo / Redo.",
)
add_task(
    "task-015",
    "tests: Implement IO-001 to IO-015 (Persistence)",
    "Implement 15 test cases for Import / Export / Persistence.",
)
add_task(
    "task-016",
    "tests: Implement INP-001 to INP-007 (Touch)",
    "Implement 7 test cases for Mobile / Touch / Stylus.",
)
add_task(
    "task-017",
    "tests: Implement PERF-001 to PERF-007 (Performance)",
    "Implement 7 test cases for Performance / Stress.",
)

# Process all tasks
run_nu(["process", session_id])
run_nu(["report", session_id])
