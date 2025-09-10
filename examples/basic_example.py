"""Hello World example using flashvm.

Minimal example showing how to execute isolated Python code inside a microVM
using flashvm.run().
"""

import flashvm as vm

code = """
print("Hello from inside flashvm!")
msg = "Hello World"
print("Message length:", len(msg))
"""

result = vm.run(code)

print("=== Host collected output ===")
print(result["stdout"].rstrip())
print(f"Exit code: {result['exit_code']}")
