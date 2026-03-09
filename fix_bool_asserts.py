import sys


def replace_lines(filepath, line_replacements):
    with open(filepath, "r") as f:
        lines = f.readlines()

    for line_num, old_str, new_str in line_replacements:
        idx = line_num - 1
        if old_str in lines[idx]:
            lines[idx] = lines[idx].replace(old_str, new_str)
        else:
            print(f"Warning: '{old_str}' not found on line {line_num}")

    with open(filepath, "w") as f:
        f.writelines(lines)


replacements = [
    (1162, "assert!(result.is_ok());", "assert!(result);"),
    (1170, "assert!(result.is_ok());", "assert!(result);"),
    (1178, "assert!(result.is_ok());", "assert!(result);"),
    (1186, "assert!(result.is_ok());", "assert!(result);"),
    (1194, "assert!(result.is_ok());", "assert!(result);"),
    (1202, "assert!(result.is_err());", "assert!(!result);"),
    (1209, "assert!(result.is_err());", "assert!(!result);"),
    (1232, "assert!(result.is_ok());", "assert!(result);"),
    (1240, "assert!(result.is_ok());", "assert!(result);"),
    (2909, "assert!(result.is_ok());", "assert!(result);"),
    (3325, "assert!(result.is_ok());", "assert!(result);"),
    (3373, "assert!(result.is_ok());", "assert!(result);"),
    (3440, "assert!(result.is_ok());", "assert!(result);"),
    (3488, "assert!(result.is_ok());", "assert!(result);"),
    (3548, "assert!(result.is_ok());", "assert!(result);"),
    (3638, "assert!(result.is_ok());", "assert!(result);"),
]

replace_lines("diagram_tool/src/ui/commands.rs", replacements)
