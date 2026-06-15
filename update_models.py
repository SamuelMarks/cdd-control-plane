import re

with open("src/db/models.rs", "r") as f:
    content = f.read()

content = content.replace("//! Database models.\n", "") # Reset if any
content = "//! Database models.\n" + content

# Replace all struct definitions with documented ones if they don't have docs
def doc_replacer(match):
    prefix = match.group(1)
    if "///" in prefix:
        return match.group(0)
    struct_name = match.group(2)
    return prefix + f"/// {struct_name} model.\n" + f"pub struct {struct_name}"

content = re.sub(r'((?:#\[.*?\]\n)*)pub struct (\w+)', doc_replacer, content)

with open("src/db/models.rs", "w") as f:
    f.write(content)
