import pytest
import qcel_howmany.rs as rs


def test_sum_as_string():
    assert rs.sum_as_string(1, 1) == "2"
