import re

with open("/home/lewis/src/seshat-e2e/e2e/test_suite_J_undo.py", "r") as f:
    content = f.read()

# For test_his_006, the nodes might not be exactly at 200,200 due to canvas offset.
# When drag_to is used, the coordinates are relative to the element (canvas).
# So the node's top left is actually canvas_x + 200, canvas_y + 200.
# We should use bounding boxes.
# Wait, in test_his_006, we did use bounding boxes!
# node1_box = nodes.nth(0).bounding_box()
# page.mouse.move(node1_box["x"] + node1_box["width"] / 2, node1_box["y"] + node1_box["height"] / 2)
# And it STILL didn't create the edge!
