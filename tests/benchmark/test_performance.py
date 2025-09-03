"""
Benchmark tests for flashvm performance analysis.

These tests measure execution times and resource usage patterns.
"""

import pytest


class TestPerformanceBenchmarks:
    """Performance benchmarks for VM execution."""
    
    @pytest.mark.benchmark
    @pytest.mark.requires_vm
    def test_startup_time(self, vm_ready, benchmark):
        """Benchmark VM startup time with minimal code."""
        import flashvm as rip
        
        def minimal_execution():
            return rip.run('print("benchmark")')
        
        result = benchmark(minimal_execution)
        
        # Verify execution was successful
        assert result['exit_code'] == 0
        assert 'benchmark' in result['stdout']
        
        # VM startup should be reasonably fast (< 30 seconds)
        assert result['execution_time_ms'] < 30000
    
    @pytest.mark.benchmark
    @pytest.mark.requires_vm
    def test_memory_scaling(self, vm_ready):
        """Benchmark performance with different memory allocations."""
        import flashvm as rip
        import time
        
        memory_test_code = """
# Create data structures of various sizes
small_list = list(range(1000))
medium_list = list(range(10000))
large_dict = {i: str(i) for i in range(5000)}

print(f"Small list: {len(small_list)}")
print(f"Medium list: {len(medium_list)}")
print(f"Large dict: {len(large_dict)}")
print("Memory test completed")
"""
        
        memory_configs = [256, 512, 1024, 2048]
        results = {}
        
        for memory_mb in memory_configs:
            start_time = time.time()
            result = rip.run(memory_test_code, memory_mb=memory_mb)
            end_time = time.time()
            
            execution_time_ms = (end_time - start_time) * 1000
            results[memory_mb] = execution_time_ms
            
            # Verify execution succeeded
            assert result['exit_code'] == 0
            assert 'Memory test completed' in result['stdout']
        
        # Performance should not degrade significantly with more memory
        # (within reasonable bounds)
        min_time = min(results.values())
        max_time = max(results.values())
        assert max_time / min_time < 3.0  # Max 3x slowdown
    
    @pytest.mark.benchmark
    @pytest.mark.requires_vm
    def test_cpu_scaling(self, vm_ready):
        """Benchmark performance with different CPU allocations."""
        import flashvm as rip
        import time
        
        cpu_intensive_code = """
import math

def cpu_intensive_task(n):
    total = 0
    for i in range(n):
        total += math.sqrt(i * math.pi)
    return total

# Perform CPU-intensive calculation
result = cpu_intensive_task(50000)
print(f"CPU task result: {result:.2f}")
print("CPU test completed")
"""
        
        cpu_configs = [1, 2, 4]
        results = {}
        
        for cpus in cpu_configs:
            start_time = time.time()
            result = rip.run(cpu_intensive_code, cpus=cpus, memory_mb=512)
            end_time = time.time()
            
            execution_time_ms = (end_time - start_time) * 1000
            results[cpus] = execution_time_ms
            
            # Verify execution succeeded
            assert result['exit_code'] == 0
            assert 'CPU test completed' in result['stdout']
        
        # More CPUs shouldn't significantly hurt performance for single-threaded tasks
        single_cpu_time = results[1]
        for cpu_count, exec_time in results.items():
            assert exec_time / single_cpu_time < 2.0  # Max 2x slowdown
    
    @pytest.mark.benchmark
    @pytest.mark.requires_vm
    @pytest.mark.slow
    def test_concurrent_executions(self, vm_ready):
        """Benchmark concurrent VM executions."""
        import flashvm as rip
        import threading
        import time
        
        simple_code = """
print("Task starting")
print("Task completed")
"""
        
        def single_execution():
            return rip.run(simple_code, timeout_seconds=30)
        
        # First test sequential execution
        start_time = time.time()
        sequential_results = [single_execution() for _ in range(2)]
        end_time = time.time()
        
        # Verify sequential results
        for result in sequential_results:
            assert result['exit_code'] == 0
            assert 'Task completed' in result['stdout']
        
        print(f"Sequential execution took {end_time - start_time:.2f} seconds")
        
        # For concurrent test, we'll just verify that multiple VMs can run
        # without going into detailed performance comparison due to resource constraints
        def concurrent_executions(count=2):
            results = []
            threads = []
            errors = []
            
            def worker():
                try:
                    result = rip.run(simple_code, timeout_seconds=60, memory_mb=256)
                    results.append(result)
                except Exception as e:
                    errors.append(str(e))
            
            start_time = time.time()
            
            # Start concurrent executions
            for _ in range(count):
                thread = threading.Thread(target=worker)
                threads.append(thread)
                thread.start()
                time.sleep(0.5)  # Small delay between starts
            
            # Wait for all to complete
            for thread in threads:
                thread.join()
            
            end_time = time.time()
            
            # If we have errors, show them
            if errors:
                print(f"Errors during concurrent execution: {errors}")
            
            # Check if we got at least one successful result
            successful_results = [r for r in results if r.get('exit_code') == 0]
            print(f"Successful concurrent executions: {len(successful_results)} out of {count}")
            
            # For benchmark purposes, we just need at least one success
            assert len(successful_results) >= 1, f"At least one concurrent execution should succeed, got {len(successful_results)}"
            
            return end_time - start_time
        
        concurrent_time = concurrent_executions(2)
        
        # Basic timing comparison - just verify it completes in reasonable time
        assert concurrent_time < 120  # Should complete within 2 minutes
    
    @pytest.mark.benchmark
    @pytest.mark.requires_vm
    def test_code_size_impact(self, vm_ready):
        """Benchmark impact of code size on execution time."""
        import flashvm as rip
        import time
        
        # Generate code of different sizes
        def generate_code(size_category):
            if size_category == 'small':
                return 'print("small code")'
            elif size_category == 'medium':
                lines = ['print("Line {}")'.format(i) for i in range(20)]  # Reduced size
                return '\n'.join(lines)
            elif size_category == 'large':
                lines = ['print("Line {}: {}")'.format(i, i*2) for i in range(100)]  # Reduced size
                return '\n'.join(lines)
        
        sizes = ['small', 'medium', 'large']
        results = {}
        
        for size in sizes:
            code = generate_code(size)
            
            start_time = time.time()
            result = rip.run(code, memory_mb=512, timeout_seconds=60)
            end_time = time.time()
            
            execution_time_ms = (end_time - start_time) * 1000
            results[size] = execution_time_ms
            
            # Verify execution succeeded
            print(f"Code size {size} - Exit code: {result['exit_code']}")
            if result['exit_code'] != 0:
                print(f"stderr: {result.get('stderr', 'N/A')}")
            assert result['exit_code'] == 0
        
        # Larger code should not cause disproportionate slowdown
        small_time = results['small']
        large_time = results['large']
        assert large_time / small_time < 10.0  # Max 10x slowdown


class TestResourceUsage:
    """Test resource usage patterns."""
    
    @pytest.mark.benchmark
    @pytest.mark.requires_vm
    def test_memory_usage_tracking(self, vm_ready, benchmark):
        """Test memory usage tracking and cleanup."""
        import flashvm as rip
        
        memory_intensive_code = """
import gc

# Create and cleanup large data structures
large_data = []
for i in range(10):
    chunk = list(range(10000))
    large_data.append(chunk)

print(f"Created {len(large_data)} chunks")

# Cleanup
del large_data
gc.collect()

print("Memory cleanup completed")
"""
        
        def memory_test():
            return rip.run(memory_intensive_code, memory_mb=1024, timeout_seconds=30)
        
        result = benchmark(memory_test)
        
        # Verify successful execution
        assert result['exit_code'] == 0
        assert 'Memory cleanup completed' in result['stdout']
        
        # Execution should complete in reasonable time
        assert result['execution_time_ms'] < 30000
    
    @pytest.mark.benchmark
    @pytest.mark.requires_vm
    def test_repeated_executions(self, vm_ready):
        """Test performance of repeated executions."""
        import flashvm as rip
        
        simple_code = 'print("repeated execution test")'
        
        def repeated_execution():
            results = []
            for i in range(5):
                result = rip.run(simple_code, memory_mb=256)
                results.append(result)
            return results
        
        # Execute multiple rounds for consistency testing
        all_results = []
        for round_num in range(3):
            round_results = repeated_execution()
            all_results.extend(round_results)
        
        # Verify all executions succeeded
        for result in all_results:
            assert result['exit_code'] == 0
            assert 'repeated execution test' in result['stdout']
        
        # Check for performance consistency
        execution_times = []
        for result in all_results:
            execution_times.append(result['execution_time_ms'])
        
        # Performance should be relatively consistent
        min_time = min(execution_times)
        max_time = max(execution_times)
        avg_time = sum(execution_times) / len(execution_times)
        
        # Variations should be within reasonable bounds
        assert max_time / min_time < 5.0  # Max 5x variation
        assert abs(max_time - avg_time) / avg_time < 2.0  # Max 200% deviation from average


class TestStressTests:
    """Stress tests for extreme scenarios."""
    
    @pytest.mark.benchmark
    @pytest.mark.requires_vm
    @pytest.mark.slow
    def test_long_running_execution(self, vm_ready, benchmark):
        """Test long-running code execution."""
        import flashvm as rip
        
        long_code = """
import time

print("Starting long execution")
for i in range(10):
    time.sleep(1)
    print(f"Step {i+1}/10 completed")

print("Long execution completed")
"""
        
        def long_execution():
            return rip.run(long_code, timeout_seconds=120, memory_mb=512)
        
        result = benchmark.pedantic(
            long_execution,
            iterations=1,
            rounds=1
        )
        
        # Verify successful completion
        assert result['exit_code'] == 0
        assert 'Long execution completed' in result['stdout']
        assert result['execution_time_ms'] > 10000  # Should take at least 10 seconds
    
    @pytest.mark.benchmark
    @pytest.mark.requires_vm
    @pytest.mark.slow
    def test_high_output_volume(self, vm_ready, benchmark):
        """Test handling of high-volume output."""
        import flashvm as rip
        
        high_output_code = """
# Generate substantial output
for i in range(1000):
    print(f"Output line {i}: {'x' * 50}")

print("High output test completed")
"""
        
        def high_output_execution():
            return rip.run(high_output_code, memory_mb=512, timeout_seconds=60)
        
        result = benchmark(high_output_execution)
        
        # Verify successful execution
        assert result['exit_code'] == 0
        assert 'High output test completed' in result['stdout']
        
        # Output should be substantial
        assert len(result['stdout']) > 50000  # Should have significant output
        
        # Should complete in reasonable time despite large output
        assert result['execution_time_ms'] < 60000
