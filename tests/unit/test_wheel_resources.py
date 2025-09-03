def test_embedded_oci_layout_check_imports():
    import flashvm as rip
    info = rip.doctor()
    assert 'offline_mode' in info


def test_embedded_oci_path_and_message_keys():
    import flashvm as rip
    info = rip.doctor()
    # Always returns keys; message optional
    assert 'krunvm' in info and 'buildah' in info and 'kvm' in info
    assert 'offline_mode' in info
    # offline_message may or may not exist
    assert isinstance(info.get('offline_mode'), bool)
