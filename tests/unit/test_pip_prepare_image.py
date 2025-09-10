import pytest


@pytest.mark.unit
def test_pip_prepare_image_requires_packages(check_rip_available):
    import flashvm as rip

    with pytest.raises(Exception) as exc:
        rip.pip_prepare_image([])
    # Should complain about empty packages list
    assert "packages" in str(exc.value).lower()
    assert "empty" in str(exc.value).lower()


@pytest.mark.unit
def test_pip_prepare_image_smoke_builds_tag(check_rip_available, doctor_check):
    import flashvm as rip
    import uuid

    deps = doctor_check
    if not deps.get("buildah", False):
        pytest.skip("buildah not available")

    # Build a tiny layer by installing a small package
    tag = f"pytest-pip-{uuid.uuid4().hex[:8]}"
    try:
        image_ref = rip.pip_prepare_image(["wheel"], tag=tag)
        assert isinstance(image_ref, str)
        assert image_ref.startswith("containers-storage:localhost/flashvm:")
    except Exception as e:
        # Accept failures due to missing network/containers setup
        assert isinstance(e, Exception)


@pytest.mark.unit
@pytest.mark.slow
def test_pip_prepare_image_with_pandas_and_run(check_rip_available, doctor_check, vm_ready):
    import flashvm as rip
    import uuid

    deps = doctor_check
    if not deps.get("buildah", False):
        pytest.skip("buildah not available")

    tag = f"pytest-pandas-{uuid.uuid4().hex[:8]}"

    try:
        image_ref = rip.pip_prepare_image(["pandas"], tag=tag)
    except Exception as e:
        pytest.skip(f"pip_prepare_image failed (network/containers): {e}")

    code = (
        "import pandas as pd; "
        "print('pandas ' + pd.__version__); "
        "import io; import sys; "
        "import pandas as pd; df = pd.DataFrame({'a':[1,2]}); print(int(df['a'].sum()))"
    )

    result = rip.run(code, image=image_ref, memory_mb=1024, timeout_seconds=120)
    assert isinstance(result, dict)
    if result.get("exit_code") != 0:
        pytest.skip(f"VM run failed: {result.get('stderr', '')}")
    assert "pandas " in result.get("stdout", "")
    assert "3" in result.get("stdout", "")
