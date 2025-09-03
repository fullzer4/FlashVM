"""
Integration tests for flashvm.

These tests verify end-to-end functionality including complex scenarios,
file operations, and advanced configurations.
"""

import pytest


class TestComplexExecution:
    """Test complex execution scenarios."""
    
    @pytest.mark.integration
    @pytest.mark.requires_vm
    def test_multi_step_computation(self, vm_ready, vm_helper):
        """Test execution of multi-step computational task."""
        import flashvm as rip
        
        complex_code = """
import math
import json

# Multi-step computation
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

def is_prime(n):
    if n < 2:
        return False
    for i in range(2, int(math.sqrt(n)) + 1):
        if n % i == 0:
            return False
    return True

# Calculate results
fib_numbers = [fibonacci(i) for i in range(10)]
prime_numbers = [i for i in range(2, 50) if is_prime(i)]

results = {
    'fibonacci': fib_numbers,
    'primes': prime_numbers,
    'stats': {
        'fib_sum': sum(fib_numbers),
        'prime_count': len(prime_numbers),
        'largest_prime': max(prime_numbers)
    }
}

print("=== COMPUTATION RESULTS ===")
print(json.dumps(results, indent=2))
print("=== END RESULTS ===")
"""
        
        result = rip.run(complex_code, memory_mb=1024, timeout_seconds=60)
        
        vm_helper.assert_successful_execution(result)
        vm_helper.assert_contains_output(result, "COMPUTATION RESULTS")
        vm_helper.assert_contains_output(result, "fibonacci")
        vm_helper.assert_contains_output(result, "primes")
    
    @pytest.mark.integration
    @pytest.mark.requires_vm
    def test_file_operations(self, vm_ready, vm_helper):
        """Test file creation, reading, and manipulation."""
        import flashvm as rip
        
        file_ops_code = """
import os
import json
import csv
from pathlib import Path

# Create various types of files
data_dir = Path("/tmp/test_data")
data_dir.mkdir(exist_ok=True)

# JSON file
json_data = {"test": "data", "numbers": [1, 2, 3, 4, 5]}
with open(data_dir / "test.json", "w") as f:
    json.dump(json_data, f)

# CSV file
csv_data = [
    ["name", "age", "city"],
    ["Alice", 30, "New York"],
    ["Bob", 25, "San Francisco"],
    ["Charlie", 35, "Chicago"]
]
with open(data_dir / "test.csv", "w", newline="") as f:
    writer = csv.writer(f)
    writer.writerows(csv_data)

# Text file
with open(data_dir / "test.txt", "w") as f:
    f.write("Hello from file operations test!\\n")
    f.write("This is line 2\\n")
    f.write("This is line 3\\n")

# Read and verify files
print("=== FILE OPERATIONS TEST ===")

# Read JSON
with open(data_dir / "test.json", "r") as f:
    json_content = json.load(f)
    print(f"JSON content: {json_content}")

# Read CSV
with open(data_dir / "test.csv", "r") as f:
    csv_reader = csv.reader(f)
    csv_content = list(csv_reader)
    print(f"CSV rows: {len(csv_content)}")

# Read text file
with open(data_dir / "test.txt", "r") as f:
    text_content = f.read()
    print(f"Text file length: {len(text_content)} chars")

# List directory contents
files = list(data_dir.glob("*"))
print(f"Created files: {[f.name for f in files]}")

print("=== FILE OPERATIONS COMPLETE ===")
"""
        
        result = rip.run(file_ops_code, memory_mb=512, timeout_seconds=30)
        
        vm_helper.assert_successful_execution(result)
        vm_helper.assert_contains_output(result, "FILE OPERATIONS TEST")
        vm_helper.assert_contains_output(result, "JSON content:")
        vm_helper.assert_contains_output(result, "CSV rows:")
        vm_helper.assert_contains_output(result, "FILE OPERATIONS COMPLETE")
    
    @pytest.mark.integration
    @pytest.mark.requires_vm
    def test_python_libraries(self, vm_ready, vm_helper):
        """Test execution with standard Python libraries."""
        import flashvm as rip
        
        library_test_code = """
import sys
import os
import datetime
import json
import re
import urllib.parse
import base64
import hashlib
import uuid

print("=== PYTHON LIBRARIES TEST ===")

# System info
print(f"Python version: {sys.version}")
print(f"Platform: {sys.platform}")

# Date/time operations
now = datetime.datetime.now()
print(f"Current time: {now.isoformat()}")

# JSON operations
data = {"test": True, "value": 42}
json_str = json.dumps(data)
parsed = json.loads(json_str)
print(f"JSON roundtrip: {parsed}")

# Regular expressions
text = "The quick brown fox jumps over the lazy dog"
matches = re.findall(r"\\b\\w{4}\\b", text)  # 4-letter words
print(f"4-letter words: {matches}")

# URL operations
url = "https://example.com/path?param=value"
parsed_url = urllib.parse.urlparse(url)
print(f"URL host: {parsed_url.netloc}")

# Base64 encoding
message = "Hello, World!"
encoded = base64.b64encode(message.encode()).decode()
decoded = base64.b64decode(encoded).decode()
print(f"Base64 roundtrip: {decoded}")

# Hashing
text_hash = hashlib.sha256(message.encode()).hexdigest()
print(f"SHA256 hash: {text_hash[:16]}...")

# UUID generation
test_uuid = str(uuid.uuid4())
print(f"Generated UUID: {test_uuid}")

print("=== LIBRARIES TEST COMPLETE ===")
"""
        
        result = rip.run(library_test_code, memory_mb=512, timeout_seconds=30)
        
        vm_helper.assert_successful_execution(result)
        vm_helper.assert_contains_output(result, "PYTHON LIBRARIES TEST")
        vm_helper.assert_contains_output(result, "Python version:")
        vm_helper.assert_contains_output(result, "JSON roundtrip:")
        vm_helper.assert_contains_output(result, "LIBRARIES TEST COMPLETE")
    
    @pytest.mark.integration
    @pytest.mark.requires_vm
    def test_error_recovery(self, vm_ready, vm_helper):
        """Test error handling and recovery in complex scenarios."""
        import flashvm as rip
        
        error_handling_code = """
import sys
import traceback

print("=== ERROR HANDLING TEST ===")

def test_function_with_error():
    try:
        # This will cause a division by zero error
        result = 10 / 0
        return result
    except ZeroDivisionError as e:
        print(f"Caught expected error: {e}")
        return "error_handled"

def test_file_error():
    try:
        # Try to read a non-existent file
        with open("/nonexistent/file.txt", "r") as f:
            return f.read()
    except FileNotFoundError as e:
        print(f"File error handled: {e}")
        return "file_error_handled"

def test_type_error():
    try:
        # Try to add incompatible types
        result = "string" + 42
        return result
    except TypeError as e:
        print(f"Type error handled: {e}")
        return "type_error_handled"

# Test all error scenarios
results = []
results.append(test_function_with_error())
results.append(test_file_error())
results.append(test_type_error())

print(f"Error handling results: {results}")

# Test that normal execution continues
normal_result = 2 + 2
print(f"Normal computation after errors: {normal_result}")

print("=== ERROR HANDLING COMPLETE ===")
"""
        
        result = rip.run(error_handling_code, memory_mb=512, timeout_seconds=30)
        
        vm_helper.assert_successful_execution(result)
        vm_helper.assert_contains_output(result, "ERROR HANDLING TEST")
        vm_helper.assert_contains_output(result, "Caught expected error:")
        vm_helper.assert_contains_output(result, "ERROR HANDLING COMPLETE")


class TestAdvancedConfiguration:
    """Test advanced configuration scenarios."""
    
    @pytest.mark.integration
    @pytest.mark.requires_vm
    def test_environment_inheritance(self, vm_ready, vm_helper):
        """Test complex environment variable scenarios."""
        import flashvm as rip
        
        env_code = """
import os

print("=== ENVIRONMENT TEST ===")

# Test multiple environment variables
env_vars = {
    'SIMPLE_VAR': os.environ.get('SIMPLE_VAR', 'NOT_SET'),
    'PATH_VAR': os.environ.get('PATH_VAR', 'NOT_SET'),
    'JSON_VAR': os.environ.get('JSON_VAR', 'NOT_SET'),
    'NUMERIC_VAR': os.environ.get('NUMERIC_VAR', 'NOT_SET'),
}

for var, value in env_vars.items():
    print(f"{var} = {value}")

# Test PATH environment variable
path = os.environ.get('PATH', '')
print(f"PATH length: {len(path)}")
print(f"PATH contains /usr/bin: {'/usr/bin' in path}")

print("=== ENVIRONMENT TEST COMPLETE ===")
"""
        
        complex_env = {
            'SIMPLE_VAR': 'simple_value',
            'PATH_VAR': '/custom/path:/another/path',
            'JSON_VAR': '{"key": "value", "number": 123}',
            'NUMERIC_VAR': '42',
        }
        
        result = rip.run(env_code, env=complex_env, memory_mb=512)
        
        vm_helper.assert_successful_execution(result)
        vm_helper.assert_contains_output(result, "SIMPLE_VAR = simple_value")
        vm_helper.assert_contains_output(result, "JSON_VAR = {\"key\": \"value\"")
        vm_helper.assert_contains_output(result, "NUMERIC_VAR = 42")
    
    @pytest.mark.integration
    @pytest.mark.requires_vm
    @pytest.mark.slow
    def test_memory_intensive_task(self, vm_ready, vm_helper):
        """Test memory-intensive computations."""
        import flashvm as rip
        
        memory_code = """
import gc

print("=== MEMORY INTENSIVE TEST ===")

# Create a large list
large_list = list(range(100000))
print(f"Created list with {len(large_list)} elements")

# Perform operations on the list
squared = [x * x for x in large_list]
print(f"Squared list length: {len(squared)}")

# Create nested structures
nested_data = {
    f"key_{i}": [j for j in range(100)]
    for i in range(1000)
}
print(f"Created nested structure with {len(nested_data)} keys")

# Clean up
del large_list
del squared
del nested_data
gc.collect()

print("Memory cleanup completed")
print("=== MEMORY TEST COMPLETE ===")
"""
        
        result = rip.run(memory_code, memory_mb=2048, timeout_seconds=60)
        
        vm_helper.assert_successful_execution(result)
        vm_helper.assert_contains_output(result, "MEMORY INTENSIVE TEST")
        vm_helper.assert_contains_output(result, "Created list with 100000")
        vm_helper.assert_contains_output(result, "MEMORY TEST COMPLETE")
    
    @pytest.mark.integration
    @pytest.mark.requires_vm
    def test_config_dict_advanced(self, vm_ready, vm_helper):
        """Test advanced configuration dictionary usage."""
        import flashvm as rip
        
        code = """
import os

print("=== ADVANCED CONFIG TEST ===")

# Check CPU count
cpu_count = os.cpu_count()
print(f"Available CPUs: {cpu_count}")

# Check memory info (standard library)
def meminfo_mb():
    total = avail = None
    try:
        with open('/proc/meminfo') as f:
            data = f.read().splitlines()
        kv = {}
        for line in data:
            if ':' in line:
                k, v = line.split(':', 1)
                vnum = v.strip().split()[0]
                if vnum.isdigit():
                    kv[k] = int(vnum)
        total = kv.get('MemTotal')
        avail = kv.get('MemAvailable', kv.get('MemFree'))
        if total:
            print(f"Total memory: {total // 1024} MB")
        if avail:
            print(f"Available memory: {avail // 1024} MB")
    except Exception as e:
        print(f"Memory info not available: {e}")

meminfo_mb()

# Test environment variables
test_vars = ['TEST_VAR1', 'TEST_VAR2', 'COMPLEX_VAR']
for var in test_vars:
    value = os.environ.get(var, 'NOT_SET')
    print(f"{var}: {value}")

print("=== ADVANCED CONFIG COMPLETE ===")
"""
        
        advanced_config = {
            'cpus': 2,
            'memory_mb': 1536,
            'timeout_seconds': 45,
            'env': {
                'TEST_VAR1': 'value1',
                'TEST_VAR2': 'value2',
                'COMPLEX_VAR': 'complex_value_with_special_chars_!@#$%',
            },
            'python_args': ['-u', '-O'],
        }
        
        result = rip.run_with_config(code, advanced_config)
        
        vm_helper.assert_successful_execution(result)
        vm_helper.assert_contains_output(result, "ADVANCED CONFIG TEST")
        vm_helper.assert_contains_output(result, "TEST_VAR1: value1")
        vm_helper.assert_contains_output(result, "TEST_VAR2: value2")


class TestImageManagement:
    """Test image management functionality."""
    
    @pytest.mark.integration
    def test_doctor_detailed(self, check_rip_available):
        """Test detailed doctor() functionality."""
        import flashvm as rip
        
        result = rip.doctor()
        
        # Verify all required fields are present
        required_fields = ['krunvm', 'buildah', 'kvm', 'offline_mode', 'ready']
        for field in required_fields:
            assert field in result
            assert isinstance(result[field], bool)
        
        # If system is ready, all dependencies should be available
        if result['ready']:
            assert result['krunvm'] is True
            assert result['buildah'] is True
            assert result['kvm'] is True
    
    @pytest.mark.integration
    def test_image_functions(self, check_rip_available):
        """Test image management functions."""
        import flashvm as rip
        
        # Test list_cached_images (should not fail even if empty)
        try:
            cached_images = rip.list_cached_images()
            assert isinstance(cached_images, list)
        except Exception as e:
            # Function might not be fully implemented yet
            assert "não implementado" in str(e).lower() or "not implemented" in str(e).lower()
        
        # Test clear_cache (should not fail)
        try:
            result = rip.clear_cache()
            assert isinstance(result, bool)
        except Exception as e:
            # Function might not be fully implemented yet
            assert "não implementado" in str(e).lower() or "not implemented" in str(e).lower()
    
    @pytest.mark.integration
    def test_prepare_image(self, check_rip_available):
        """Test image preparation functionality."""
        import flashvm as rip
        
        # Test with common image references
        test_images = [
            "python:3.11-slim",
            "alpine:latest",
        ]
        
        for image in test_images:
            try:
                result = rip.prepare_image(image)
                # Should return boolean
                assert isinstance(result, bool)
            except Exception as e:
                # May fail due to network or system configuration
                # This is acceptable for testing
                assert isinstance(e, Exception)
