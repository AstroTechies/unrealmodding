import re
import sys

dependencyPattern = re.compile(r"^([a-zA-Z0-9_]*) = { path = \"([./a-zA-Z0-9_]*)\", version = \"([.a-zA-Z0-9_]*)\" }", re.MULTILINE)
versionPattern = re.compile(r"^version = \"([.a-zA-Z0-9_]*)\"", re.MULTILINE)

f = open(sys.argv[1], "r")
data = f.read()

version = versionPattern.findall(data)[0]

def replace(match):
    return f'{match.group(1)} = {{ path = "{match.group(2)}", version = "{version}" }}'

newData = re.sub(dependencyPattern, replace, data)
open(sys.argv[1], "w").write(newData)
