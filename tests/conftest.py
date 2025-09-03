"""
Pytest configuration and shared fixtures for flashvm tests.
"""

import pytest
from typing import Dict, Any

try:
    import flashvm as rip
    RIP_AVAILABLE = True
except ImportError:
    RIP_AVAILABLE = False


@pytest.fixture(scope="session")
def check_rip_available():
    """Check if flashvm module is available."""
    if not RIP_AVAILABLE:
        pytest.skip("flashvm module not available")
    return True


@pytest.fixture(scope="session")
def doctor_check():
    """Check system dependencies using doctor() function."""
    if not RIP_AVAILABLE:
        pytest.skip("flashvm module not available")
    
    try:
        deps = rip.doctor()
        return deps
    except Exception as e:
        pytest.skip(f"Doctor check failed: {e}")


@pytest.fixture
def vm_ready(doctor_check):
    """Check if VM execution is ready."""
    deps = doctor_check
    if not deps.get('ready', False):
        missing = []
        if not deps.get('krunvm', False):
            missing.append('krunvm')
        if not deps.get('buildah', False):
            missing.append('buildah')
        if not deps.get('kvm', False):
            missing.append('kvm')
        
        pytest.skip(f"VM execution not ready. Missing: {', '.join(missing)}")
    
    return deps


@pytest.fixture
def sample_python_code():
    """Simple Python code for testing."""
    return """
print("Hello from microVM!")
print("Testing basic execution")
result = 2 + 2
print(f"2 + 2 = {result}")
"""


@pytest.fixture
def env_test_code():
    """Python code that tests environment variables."""
    return """
import os
print("Environment variables test:")
print(f"CUSTOM_VAR = {os.environ.get('CUSTOM_VAR', 'NOT_SET')}")
print(f"TEST_ENV = {os.environ.get('TEST_ENV', 'NOT_SET')}")
print(f"HOME = {os.environ.get('HOME', 'NOT_SET')}")
"""


@pytest.fixture
def math_computation_code():
    """Python code that performs mathematical computations."""
    return """
import math
import statistics

# Basic math operations
numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
result = {
    'sum': sum(numbers),
    'mean': statistics.mean(numbers),
    'median': statistics.median(numbers),
    'sqrt_of_100': math.sqrt(100),
    'pi': math.pi
}

for key, value in result.items():
    print(f"{key}: {value}")
"""


@pytest.fixture
def file_operations_code():
    """Python code that performs file operations."""
    return """
import os
import tempfile

# Test file operations
test_content = "Hello from microVM file test!"

# Write to a file
with open('/tmp/test_file.txt', 'w') as f:
    f.write(test_content)

# Read from the file
with open('/tmp/test_file.txt', 'r') as f:
    content = f.read()

print(f"File content: {content}")
print(f"File exists: {os.path.exists('/tmp/test_file.txt')}")

# List current directory
print("Current directory contents:")
for item in os.listdir('.'):
    print(f"  {item}")
"""


@pytest.fixture
def basic_vm_config():
    """Basic VM configuration for testing."""
    return {
        'cpus': 1,
        'memory_mb': 512,
        'timeout_seconds': 30,
        'network': False,
    }


@pytest.fixture
def extended_vm_config():
    """Extended VM configuration with environment variables."""
    return {
        'cpus': 2,
        'memory_mb': 1024,
        'timeout_seconds': 60,
        'network': False,
        'env': {
            'CUSTOM_VAR': 'test_value',
            'TEST_ENV': 'pytest_environment',
            'PYTHON_ENV': 'testing'
        }
    }


@pytest.fixture
def performance_config():
    """Configuration optimized for performance testing."""
    return {
        'cpus': 4,
        'memory_mb': 2048,
        'timeout_seconds': 120,
        'network': False,
    }


class VMExecutionHelper:
    """Helper class for VM execution testing."""
    
    @staticmethod
    def assert_successful_execution(result: Dict[str, Any]):
        """Assert that a VM execution was successful."""
        assert 'stdout' in result
        assert 'stderr' in result
        assert 'exit_code' in result
        assert 'execution_time_ms' in result
        assert 'image_used' in result
        assert result['exit_code'] == 0
        assert isinstance(result['execution_time_ms'], (int, float))
        assert result['execution_time_ms'] > 0
    
    @staticmethod
    def assert_contains_output(result: Dict[str, Any], expected_text: str):
        """Assert that the execution output contains expected text."""
        assert expected_text in result['stdout']
    
    @staticmethod
    def assert_execution_time_reasonable(result: Dict[str, Any], max_time_ms: int = 30000):
        """Assert that execution time is reasonable."""
        exec_time = result['execution_time_ms']
        assert exec_time < max_time_ms, f"Execution took too long: {exec_time}ms"


@pytest.fixture
def vm_helper():
    """Fixture providing VM execution helper methods."""
    return VMExecutionHelper()


@pytest.fixture
def temp_test_dir(tmp_path):
    """Create a temporary directory for test files."""
    test_dir = tmp_path / "rodar_isolado_tests"
    test_dir.mkdir()
    return test_dir


def pytest_configure(config):
    """Configure pytest with custom markers."""
    config.addinivalue_line(
        "markers", "slow: mark test as slow running"
    )
    config.addinivalue_line(
        "markers", "integration: mark test as integration test"
    )
    config.addinivalue_line(
        "markers", "unit: mark test as unit test"
    )
    config.addinivalue_line(
        "markers", "benchmark: mark test as benchmark test"
    )
    config.addinivalue_line(
        "markers", "requires_vm: mark test as requiring VM capabilities"
    )


def pytest_collection_modifyitems(config, items):
    """Automatically mark tests based on their location."""
    for item in items:
        # Mark tests in integration directory
        if "integration" in str(item.fspath):
            item.add_marker(pytest.mark.integration)
            item.add_marker(pytest.mark.requires_vm)
        
        # Mark tests in benchmark directory
        if "benchmark" in str(item.fspath):
            item.add_marker(pytest.mark.benchmark)
            item.add_marker(pytest.mark.slow)
        
        # Mark tests in unit directory
        if "unit" in str(item.fspath):
            item.add_marker(pytest.mark.unit)
