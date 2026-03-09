import sys

def resolve_file(filepath):
    with open(filepath, 'r') as f:
        lines = f.readlines()

    out = []
    in_conflict = False
    in_diff = False

    for line in lines:
        if line.startswith('<<<<<<< conflict'):
            in_conflict = True
            in_diff = False
            continue
        elif line.startswith('+++++++'):
            continue
        elif line.startswith('%%%%%%% diff from:'):
            in_diff = True
            continue
        elif line.startswith('\\\\\\\\\\\\\\ to:'):
            continue
        elif line.startswith('>>>>>>> conflict'):
            in_conflict = False
            in_diff = False
            continue
        
        if in_conflict and in_diff:
            if line.startswith('-'):
                continue # Skip removed lines
            elif line.startswith('+'):
                out.append(line[1:]) # Keep added lines, remove '+'
            elif line.startswith(' '):
                out.append(line[1:]) # Keep context lines, remove space
            elif line == '\n':
                out.append(line)
            else:
                out.append(line)
        elif not in_conflict:
            out.append(line)

    with open(filepath, 'w') as f:
        f.writelines(out)

resolve_file('diagram_tool/src/ui/commands.rs')
resolve_file('.beads/seshat-6sl/implementation.md')
