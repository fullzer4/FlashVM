#!/usr/bin/env python3
"""
Basic example of using the flashvm module

This script demonstrates how to use the library to run Python code
in isolated microVMs using libkrun.
"""

import flashvm as vm

def main():
    """Basic execution demo"""
    print("=== flashvm demonstration ===\n")

    print("=== Basic Test ===")
    
    code = """
import sys
print(f"Python version: {sys.version}")
print("Hello from flashvm!")
print("Calculating 2 + 2 =", 2 + 2)

# Demonstrate some built-in modules
import math
print(f"Pi = {math.pi:.4f}")
print(f"Square root of 16 = {math.sqrt(16)}")

import datetime
now = datetime.datetime.now()
print(f"Current datetime: {now.strftime('%Y-%m-%d %H:%M:%S')}")
"""
    
    try:
        result = vm.run(code)
        if result['exit_code'] == 0:
            print("✅ Execution succeeded!")
        else:
            print("❌ Execution failed")
        print("Stdout:\n")
        print(result['stdout'])
        if result.get('stderr'):
            print("Stderr:\n")
            print(result['stderr'])
        print(f"Exit code: {result['exit_code']}")
        print(f"Execution time: {result['execution_time_ms']} ms")
        print(f"Image used: {result.get('image_used', '<unknown>')}")
    except Exception as e:
        print(f"❌ Error: {e}")

if __name__ == "__main__":
    main()
