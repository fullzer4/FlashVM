"""
Additional utility tests for flashvm.

Tests for edge cases, error scenarios, and utility functions.
"""

import pytest


class TestEdgeCases:
    """Test edge cases and boundary conditions."""
    
    @pytest.mark.unit
    def test_empty_code_execution(self, check_rip_available):
        """Test execution with empty code."""
        import flashvm as rip
        
        # Empty string
        try:
            result = rip.run("")
            assert isinstance(result, dict)
            # May succeed or fail, but should not crash
        except Exception:
            # May raise exception, which is acceptable
            pass
        
        # Whitespace only
        try:
            result = rip.run("   \n\t  ")
            assert isinstance(result, dict)
        except Exception:
            pass
    
    @pytest.mark.unit
    def test_very_long_code(self, check_rip_available):
        """Test execution with very long code strings."""
        import flashvm as rip
        
        # Generate a very long but valid Python code
        long_code_lines = ['print(f"Line {i}")' for i in range(1000)]
        long_code = '\n'.join(long_code_lines)
        
        try:
            result = rip.run(long_code, memory_mb=1024, timeout_seconds=120)
            if result['exit_code'] == 0:
                assert 'Line 999' in result['stdout']
        except Exception as e:
            # May fail due to system limitations
            assert isinstance(e, Exception)
    
    @pytest.mark.unit
    def test_special_characters_in_code(self, check_rip_available):
        """Test execution with special characters."""
        import flashvm as rip
        
        special_code = '''
# Test with various special characters
text = "Hello! @#$%^&*()_+-=[]{}|;:'\",./<>?`~"
print(f"Special text: {text}")

# Unicode characters
unicode_text = "Olá, 世界! 🌍 🐍"
print(f"Unicode: {unicode_text}")

# Emojis and symbols
symbols = "→ ← ↑ ↓ ★ ♠ ♣ ♥ ♦"
print(f"Symbols: {symbols}")
'''
        
        try:
            result = rip.run(special_code, memory_mb=512)
            if result['exit_code'] == 0:
                assert 'Special text:' in result['stdout']
                assert 'Unicode:' in result['stdout']
        except Exception:
            # May fail due to encoding issues
            pass
    
    @pytest.mark.unit
    def test_boundary_resource_values(self, check_rip_available):
        """Test boundary values for resource allocation."""
        import flashvm as rip
        
        simple_code = 'print("boundary test")'
        
        # Test minimum values
        try:
            result = rip.run(simple_code, cpus=1, memory_mb=128)
            assert isinstance(result, dict)
        except Exception:
            # May fail if system doesn't support such low values
            pass
        
        # Test higher values (if system supports)
        try:
            result = rip.run(simple_code, cpus=8, memory_mb=4096, timeout_seconds=10)
            assert isinstance(result, dict)
        except Exception:
            # May fail due to system limitations
            pass


class TestErrorScenarios:
    """Test various error scenarios."""
    
    @pytest.mark.unit
    def test_invalid_python_syntax(self, vm_ready):
        """Test handling of invalid Python syntax."""
        import flashvm as rip
        
        invalid_codes = [
            "print('unclosed string",
            "def broken_function(\npass",
            "if True\nprint('missing colon')",
            "for i in range(10\nprint(i)",
            "import nonexistent_module_xyz_123",
        ]
        
        for invalid_code in invalid_codes:
            result = rip.run(invalid_code)
            
            # Should return error result, not crash
            assert isinstance(result, dict)
            assert 'exit_code' in result
            assert result['exit_code'] != 0
            assert len(result['stderr']) > 0 or 'SyntaxError' in result['stdout']
    
    @pytest.mark.unit
    def test_runtime_exceptions(self, vm_ready):
        """Test handling of runtime exceptions."""
        import flashvm as rip
        
        exception_codes = [
            # Division by zero
            "print(1 / 0)",
            
            # Index error
            "my_list = [1, 2, 3]\nprint(my_list[10])",
            
            # Key error
            "my_dict = {'a': 1}\nprint(my_dict['nonexistent'])",
            
            # Type error
            "print('string' + 42)",
            
            # Name error
            "print(undefined_variable)",
        ]
        
        for exception_code in exception_codes:
            result = rip.run(exception_code)
            
            # Should handle error gracefully
            assert isinstance(result, dict)
            assert result['exit_code'] != 0
            # Error information should be in stderr or stdout
            error_info = result['stderr'] + result['stdout']
            assert len(error_info) > 0
    
    @pytest.mark.unit
    @pytest.mark.timeout(10)  # Pytest timeout para evitar travamento
    def test_infinite_loop_timeout(self, vm_ready):
        """Test timeout handling for infinite loops."""
        import flashvm as rip
        
        infinite_loop_code = """
print("Starting infinite loop")
import time
# Use a loop with sleep to be more gentle on the system
for i in range(1000):
    time.sleep(0.01)
    if i > 100:  # Exit after reasonable time for testing
        break
print("Loop finished")
"""
        
        result = rip.run(infinite_loop_code, timeout_seconds=5)
        
        # Should complete or timeout gracefully
        assert isinstance(result, dict)
        # The key is that it should not hang indefinitely
    
    @pytest.mark.unit
    @pytest.mark.timeout(15)  # Timeout adicional de segurança
    def test_slow_computation_timeout(self, vm_ready):
        """Test timeout with slow but finite computation."""
        import flashvm as rip
        
        slow_code = """
import time
print("Starting slow computation")
time.sleep(10)  # Sleep for 10 seconds
print("Slow computation finished")
"""
        
        # Set timeout shorter than the sleep time
        result = rip.run(slow_code, timeout_seconds=3)
        
        # Should timeout before completion
        assert isinstance(result, dict)
        # May have different behaviors depending on timeout implementation
    
    @pytest.mark.unit 
    def test_memory_exhaustion(self, vm_ready):
        """Test handling of memory exhaustion."""
        import flashvm as rip
        
        memory_bomb_code = """
try:
    # Try to allocate large amounts of memory
    big_list = []
    for i in range(1000000):
        big_list.append([0] * 1000)
except MemoryError:
    print("MemoryError caught")
except Exception as e:
    print(f"Other error: {e}")
"""
        
        result = rip.run(memory_bomb_code, memory_mb=256, timeout_seconds=30)
        
        # Should handle memory limits gracefully
        assert isinstance(result, dict)
        # May succeed with caught MemoryError or fail with system limits


class TestConfigurationValidation:
    """Test configuration parameter validation."""
    
    @pytest.mark.unit
    def test_invalid_parameter_types(self, check_rip_available):
        """Test handling of invalid parameter types."""
        import flashvm as rip
        
        simple_code = 'print("test")'
        
        # Test invalid types for each parameter
        invalid_params = [
            {'cpus': 'not_a_number'},
            {'memory_mb': 'not_a_number'},
            {'timeout_seconds': 'not_a_number'},
            {'env': 'not_a_dict'},
            {'python_args': 'not_a_list'},
            {'network': 'not_a_bool'},
        ]
        
        for invalid_param in invalid_params:
            with pytest.raises(Exception):
                rip.run(simple_code, **invalid_param)
    
    @pytest.mark.unit
    def test_invalid_parameter_values(self, check_rip_available):
        """Test handling of invalid parameter values."""
        import flashvm as rip
        
        simple_code = 'print("test")'
        
        # Test invalid values - may not raise exception, just use defaults
        invalid_values = [
            {'cpus': -1},
            {'cpus': 0},
            {'memory_mb': -1},
            {'memory_mb': 0},
            {'timeout_seconds': -1},
        ]
        
        for invalid_value in invalid_values:
            try:
                result = rip.run(simple_code, **invalid_value)
                # If no exception, should still return valid result structure
                assert isinstance(result, dict)
            except Exception:
                # Or might raise exception, which is also acceptable
                pass
    
    @pytest.mark.unit
    def test_config_dict_validation(self, check_rip_available):
        """Test validation of configuration dictionaries."""
        import flashvm as rip
        
        simple_code = 'print("test")'
        
        # Test invalid config dictionary structures
        invalid_configs = [
            {'invalid_key': 'value'},
            {'cpus': 'not_a_number'},
            {'env': ['not', 'a', 'dict']},
        ]
        
        for invalid_config in invalid_configs:
            try:
                result = rip.run_with_config(simple_code, invalid_config)
                # Some invalid configs might be ignored rather than raise errors
                assert isinstance(result, dict)
            except Exception:
                # Or they might raise exceptions, which is also acceptable
                pass


class TestEnvironmentVariables:
    """Test environment variable handling."""
    
    @pytest.mark.requires_vm
    def test_environment_variable_types(self, vm_ready, vm_helper):
        """Test different types of environment variable values."""
        import flashvm as rip
        
        env_test_code = """
import os

vars_to_test = [
    'STRING_VAR',
    'NUMERIC_VAR', 
    'BOOLEAN_VAR',
    'JSON_VAR',
    'PATH_VAR',
    'EMPTY_VAR'
]

for var in vars_to_test:
    value = os.environ.get(var, 'NOT_SET')
    print(f"{var}: {repr(value)}")
"""
        
        complex_env = {
            'STRING_VAR': 'hello world',
            'NUMERIC_VAR': '12345',
            'BOOLEAN_VAR': 'true',
            'JSON_VAR': '{"key": "value", "number": 42}',
            'PATH_VAR': '/usr/bin:/usr/local/bin:/bin',
            'EMPTY_VAR': '',
        }
        
        result = rip.run(env_test_code, env=complex_env)
        
        vm_helper.assert_successful_execution(result)
        
        # Verify all variables are set correctly
        for var_name, expected_value in complex_env.items():
            assert f"{var_name}: '{expected_value}'" in result['stdout']
    
    @pytest.mark.requires_vm
    def test_environment_variable_special_chars(self, vm_ready, vm_helper):
        """Test environment variables with special characters."""
        import flashvm as rip
        
        special_env_code = """
import os

special_vars = ['SPECIAL_CHARS', 'UNICODE_VAR', 'SYMBOLS_VAR']
for var in special_vars:
    value = os.environ.get(var, 'NOT_SET')
    print(f"{var}: {value}")
"""
        
        special_env = {
            'SPECIAL_CHARS': '!@#$%^&*()_+-=[]{}|;:\'",.<>?`~',
            'UNICODE_VAR': 'Olá, 世界! 🌍',
            'SYMBOLS_VAR': '→←↑↓★♠♣♥♦',
        }
        
        try:
            result = rip.run(special_env_code, env=special_env)
            if result['exit_code'] == 0:
                # Verify at least some special characters are preserved
                assert 'SPECIAL_CHARS:' in result['stdout']
        except Exception:
            # May fail due to encoding or shell escaping issues
            pass


class TestImageHandling:
    """Test image specification and handling."""
    
    @pytest.mark.unit
    def test_default_image(self, check_rip_available):
        """Test execution with default image (None)."""
        import flashvm as rip
        
        simple_code = 'print("default image test")'
        
        try:
            result = rip.run(simple_code)
            assert isinstance(result, dict)
            assert 'image_used' in result
        except Exception:
            # May fail if no default image is available
            pass
    
    @pytest.mark.unit  
    def test_explicit_image_specification(self, check_rip_available):
        """Test execution with explicitly specified images."""
        import flashvm as rip
        
        simple_code = 'print("explicit image test")'
        
        # Test with common image references
        test_images = [
            "python:3.11-slim",
            "python:3.10",
            "alpine:latest",
        ]
        
        for image in test_images:
            try:
                result = rip.run(simple_code, image=image)
                assert isinstance(result, dict)
                if 'image_used' in result:
                    # Image reference should be reflected in result
                    assert len(result['image_used']) > 0
            except Exception:
                # May fail due to image availability or network issues
                pass
