"""Driver 层单元测试 — 不依赖 GUI，测试纯逻辑部分。"""

import os
import tempfile

import pytest
from PIL import Image


def test_rect_center():
    from driver.window import Rect
    r = Rect(100, 200, 400, 300)
    assert r.center == (300, 350)
    assert r.right == 500
    assert r.bottom == 500


def test_rect_zero():
    from driver.window import Rect
    r = Rect(0, 0, 0, 0)
    assert r.center == (0, 0)


def test_compare_images_identical():
    """两张相同图片差异为 0。"""
    from driver.image_match import compare_images

    with tempfile.TemporaryDirectory() as tmpdir:
        img = Image.new("RGB", (100, 100), color=(255, 0, 0))
        path_a = os.path.join(tmpdir, "a.png")
        path_b = os.path.join(tmpdir, "b.png")
        img.save(path_a)
        img.save(path_b)

        diff_pct, diff_img = compare_images(path_a, path_b)
        assert diff_pct == 0.0
        assert diff_img is not None


def test_compare_images_different():
    """黑白两张图差异应 > 0。"""
    from driver.image_match import compare_images

    with tempfile.TemporaryDirectory() as tmpdir:
        white = Image.new("RGB", (100, 100), color=(255, 255, 255))
        black = Image.new("RGB", (100, 100), color=(0, 0, 0))
        path_a = os.path.join(tmpdir, "white.png")
        path_b = os.path.join(tmpdir, "black.png")
        white.save(path_a)
        black.save(path_b)

        diff_pct, diff_img = compare_images(path_a, path_b)
        assert diff_pct > 0.9  # 完全不同应接近 1.0


def test_compare_images_different_sizes():
    """不同尺寸的图片应自动 resize 后对比。"""
    from driver.image_match import compare_images

    with tempfile.TemporaryDirectory() as tmpdir:
        img_a = Image.new("RGB", (100, 100), color=(128, 128, 128))
        img_b = Image.new("RGB", (200, 200), color=(128, 128, 128))
        path_a = os.path.join(tmpdir, "small.png")
        path_b = os.path.join(tmpdir, "large.png")
        img_a.save(path_a)
        img_b.save(path_b)

        diff_pct, _ = compare_images(path_a, path_b)
        assert diff_pct < 0.01  # 颜色相同，差异应极小


def test_image_not_found_error():
    from driver.image_match import ImageNotFoundError
    with pytest.raises(ImageNotFoundError):
        raise ImageNotFoundError("test")


def test_sleep():
    """验证 sleep 是可调用的。"""
    import driver
    import time
    start = time.time()
    driver.sleep(0.05)
    elapsed = time.time() - start
    assert elapsed >= 0.04
