import pytest
import quclif.rs as rs


def test_sum_as_string():
    assert rs.sum_as_string(1, 1) == "2"
