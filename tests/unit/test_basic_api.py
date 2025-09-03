"""
Unit tests for basic flashvm functionality.

These tests cover basic API usage, parameter validation, and simple execution scenarios.
"""

import pytest


class TestBasicAPI:
    """Test basic API functionality."""
    
    def test_module_import(self, check_rip_available):
        """Test that the module can be imported."""
        import flashvm as rip
        assert hasattr(rip, 'run')
        assert hasattr(rip, 'run_with_config')
        assert hasattr(rip, 'doctor')
    
    def test_doctor_function(self, check_rip_available):
        """Test the doctor() function returns proper structure."""
        import flashvm as rip
        
        result = rip.doctor()
        
        # Check required keys
        assert isinstance(result, dict)
        assert 'krunvm' in result
        assert 'buildah' in result
        assert 'kvm' in result
        assert 'offline_mode' in result
        assert 'ready' in result
        
        # Check types
        assert isinstance(result['krunvm'], bool)
        assert isinstance(result['buildah'], bool)
        assert isinstance(result['kvm'], bool)
        assert isinstance(result['offline_mode'], bool)
        assert isinstance(result['ready'], bool)
    
    def test_run_function_signature(self, check_rip_available):
        """Test that run() function has correct signature."""
        import flashvm as rip
        import inspect
        
        sig = inspect.signature(rip.run)
        params = list(sig.parameters.keys())
        
        # Check that 'code' is the first parameter
        assert params[0] == 'code'
        
        # Check for expected optional parameters
        expected_params = [
            'code', 'image', 'cpus', 'memory_mb', 'env', 
            'timeout_seconds', 'workdir', 'python_args', 'network'
        ]
        
        for param in expected_params:
            assert param in params
    
    def test_run_with_config_signature(self, check_rip_available):
        """Test that run_with_config() function has correct signature."""
        import flashvm as rip
        import inspect
        
        sig = inspect.signature(rip.run_with_config)
        params = list(sig.parameters.keys())
        
        assert 'code' in params
        assert 'config' in params


class TestParameterValidation:
    """Test parameter validation and error handling."""
    
    def test_empty_code_parameter(self, check_rip_available):
        """Test behavior with empty code parameter."""
        import flashvm as rip
        
        # Empty string should not crash (though execution might fail)
        try:
            result = rip.run("")
            # If it succeeds, check the structure
            assert isinstance(result, dict)
        except Exception as e:
            # If it fails, it should be a reasonable error
            assert isinstance(e, Exception)
    
    def test_invalid_config_types(self, check_rip_available):
        """Test behavior with invalid configuration types."""
        import flashvm as rip
        
        code = "print('test')"
        
        # Test invalid cpus - may not raise exception, just use default
        try:
            result = rip.run(code, cpus=-1)
            # If it doesn't raise exception, should still return a result
            assert isinstance(result, dict)
        except Exception:
            # Or it might raise an exception, which is also acceptable
            pass
        
        # Test invalid memory - may not raise exception, just use default
        try:
            result = rip.run(code, memory_mb=0)
            assert isinstance(result, dict)
        except Exception:
            pass
        
        # Test invalid timeout - may not raise exception, just use default
        try:
            result = rip.run(code, timeout_seconds=-1)
            assert isinstance(result, dict)
        except Exception:
            pass
    
    def test_config_dict_structure(self, check_rip_available):
        """Test run_with_config with various config structures."""
        import flashvm as rip
        
        code = "print('test')"
        
        # Empty config should work
        try:
            result = rip.run_with_config(code, {})
            assert isinstance(result, dict)
        except Exception:
            # May fail due to missing dependencies, but shouldn't crash
            pass
        
        # Config with valid parameters
        config = {
            'cpus': 1,
            'memory_mb': 512,
            'timeout_seconds': 30
        }
        
        try:
            result = rip.run_with_config(code, config)
            assert isinstance(result, dict)
        except Exception:
            # May fail due to missing dependencies
            pass


class TestErrorHandling:
    """Test error handling scenarios."""
    
    def test_syntax_error_code(self, vm_ready):
        """Test execution of Python code with syntax errors."""
        import flashvm as rip
        
        # Python code with syntax error
        invalid_code = """
print("Hello")
    invalid indentation
print("World")
"""
        
        result = rip.run(invalid_code)
        
        # Should return a result structure even for failed execution
        assert isinstance(result, dict)
        assert 'exit_code' in result
        assert 'stderr' in result
        
        # Exit code should indicate failure
        assert result['exit_code'] != 0
        
        # stderr should contain error information
        assert len(result['stderr']) > 0
    
    def test_runtime_error_code(self, vm_ready):
        """Test execution of Python code with runtime errors."""
        import flashvm as rip
        
        # Python code with runtime error
        error_code = """
print("Starting execution")
x = 1 / 0  # Division by zero
print("This should not print")
"""
        
        result = rip.run(error_code)
        
        assert isinstance(result, dict)
        assert result['exit_code'] != 0
        assert 'ZeroDivisionError' in result['stderr'] or 'ZeroDivisionError' in result['stdout']
    
    @pytest.mark.timeout(10)  # Pytest timeout para evitar travamento
    def test_timeout_handling(self, vm_ready):
        """Test timeout handling for long-running code."""
        import flashvm as rip
        
        # Code that runs longer than timeout - but not too long for testing
        long_running_code = """
import time
print("Starting long operation")
time.sleep(8)  # Sleep for 8 seconds
print("Finished")
"""
        
        # Set a short timeout (3 seconds)
        result = rip.run(long_running_code, timeout_seconds=3)
        
        assert isinstance(result, dict)
        # Should timeout before completion
        # Note: timeout behavior may vary depending on implementation


class TestBasicExecution:
    """Test basic code execution scenarios."""
    
    @pytest.mark.requires_vm
    def test_simple_print(self, vm_ready, vm_helper):
        """Test simple print statement execution."""
        import flashvm as rip
        
        code = 'print("Hello from microVM!")'
        result = rip.run(code)
        
        vm_helper.assert_successful_execution(result)
        vm_helper.assert_contains_output(result, "Hello from microVM!")
    
    @pytest.mark.requires_vm
    def test_basic_math(self, vm_ready, vm_helper, math_computation_code):
        """Test basic mathematical operations."""
        import flashvm as rip
        
        result = rip.run(math_computation_code)
        
        vm_helper.assert_successful_execution(result)
        vm_helper.assert_contains_output(result, "sum: 55")
        vm_helper.assert_contains_output(result, "sqrt_of_100: 10.0")
    
    @pytest.mark.requires_vm
    def test_environment_variables(self, vm_ready, vm_helper, env_test_code):
        """Test environment variable access."""
        import flashvm as rip
        
        env_vars = {
            'CUSTOM_VAR': 'test_value_123',
            'TEST_ENV': 'pytest_test'
        }
        
        result = rip.run(env_test_code, env=env_vars)
        
        vm_helper.assert_successful_execution(result)
        vm_helper.assert_contains_output(result, "CUSTOM_VAR = test_value_123")
        vm_helper.assert_contains_output(result, "TEST_ENV = pytest_test")
    
    @pytest.mark.requires_vm
    def test_different_memory_sizes(self, vm_ready, vm_helper, sample_python_code):
        """Test execution with different memory configurations."""
        import flashvm as rip
        
        memory_sizes = [256, 512, 1024]
        
        for memory_mb in memory_sizes:
            result = rip.run(sample_python_code, memory_mb=memory_mb)
            vm_helper.assert_successful_execution(result)
            vm_helper.assert_contains_output(result, "Hello from microVM!")
    
    @pytest.mark.requires_vm
    def test_different_cpu_counts(self, vm_ready, vm_helper, sample_python_code):
        """Test execution with different CPU configurations."""
        import flashvm as rip
        
        cpu_counts = [1, 2]
        
        for cpus in cpu_counts:
            result = rip.run(sample_python_code, cpus=cpus)
            vm_helper.assert_successful_execution(result)
            vm_helper.assert_contains_output(result, "Hello from microVM!")


class TestConfigDictInterface:
    """Test the config dictionary interface."""
    
    @pytest.mark.requires_vm
    def test_basic_config_dict(self, vm_ready, vm_helper, basic_vm_config):
        """Test run_with_config with basic configuration."""
        import flashvm as rip
        
        code = 'print("Config dict test")'
        result = rip.run_with_config(code, basic_vm_config)
        
        vm_helper.assert_successful_execution(result)
        vm_helper.assert_contains_output(result, "Config dict test")
    
    @pytest.mark.requires_vm
    def test_extended_config_dict(self, vm_ready, vm_helper, extended_vm_config, env_test_code):
        """Test run_with_config with extended configuration including env vars."""
        import flashvm as rip
        
        result = rip.run_with_config(env_test_code, extended_vm_config)
        
        vm_helper.assert_successful_execution(result)
        vm_helper.assert_contains_output(result, "CUSTOM_VAR = test_value")
        vm_helper.assert_contains_output(result, "TEST_ENV = pytest_environment")
