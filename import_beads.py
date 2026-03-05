import json
import subprocess
import os
import sys


def import_issues(filepath):
    if not os.path.exists(filepath):
        print(f"File not found: {filepath}")
        return

    with open(filepath, "r") as f:
        for line in f:
            if not line.strip():
                continue

            try:
                issue = json.loads(line)
            except json.JSONDecodeError as e:
                print(f"JSON error: {e} for line: {line[:50]}")
                continue

            title = issue.get("title", "Untitled")
            desc = issue.get("description", "")
            if not desc:
                desc = "No description provided."
            issue_type = issue.get("issue_type", "task")
            if issue_type not in ["bug", "feature", "task", "epic", "chore"]:
                issue_type = "task"
            prio = issue.get("priority", 2)
            if prio is None:
                prio = 2

            # create issue
            cmd = [
                "bd",
                "create",
                title,
                "--description",
                desc,
                "-t",
                issue_type,
                "-p",
                str(prio),
                "--json",
            ]
            print(f"Creating: {title}")
            res = subprocess.run(cmd, capture_output=True, text=True)
            if res.returncode != 0:
                print(f"Error creating: {res.stderr}")
                continue

            # parse out the new ID
            try:
                out = json.loads(res.stdout)
                new_id = out.get("id")
            except json.JSONDecodeError:
                # If bd output isn't clean json
                print(f"Could not parse json output from bd create: {res.stdout}")
                continue

            # check if it needs to be closed
            status = issue.get("status", "open")
            if status == "closed":
                close_reason = issue.get("close_reason", "Imported as closed")
                if not close_reason:
                    close_reason = "Imported as closed"
                print(f"Closing: {new_id}")
                subprocess.run(
                    ["bd", "close", new_id, "--reason", close_reason, "--json"],
                    capture_output=True,
                )


if __name__ == "__main__":
    files_to_import = [".beads/.beads/issues.jsonl", ".beads/backup/issues.jsonl"]
    for f in files_to_import:
        print(f"--- Importing {f} ---")
        import_issues(f)
