import quclif
import hashlib
import json
import pathlib
import types
from typing import get_args, get_origin
# from pydantic import BaseModel
import inspect
import os
from typing import get_args,  get_origin

def hash_dict_no_order(dictionary):
    encoded = json.dumps(dictionary, sort_keys=True).encode()
    return hashlib.md5(encoded).hexdigest()

def dump_data(data):
    # if isinstance(data, BaseModel):
    #     return data.model_dump()
    if isinstance(data, quclif.IdentityCirc):
        return data.pythonize()
    elif type(data) == list:
        return [dump_data(item) for item in data]
    elif type(data) == dict:
        return {dump_data(k): dump_data(v) for k, v in data.items()}
    elif type(data) == tuple:
        return list(data)
    elif type(data) == int:
        return data
    elif type(data) == float:
        return data
    elif type(data) == str:
        return data
    elif type(data) == bool:
        return data
    elif data is None:
        return '__none__'
    else:
        raise TypeError(f"Unsupported type: {type(data)}")

def load_data(data, returnty):
    # if issubclass(returnty, BaseModel):
    #     return returnty.model_validate(data)
    if get_origin(returnty) == quclif.IdentityCirc:
        return quclif.IdentityCirc.from_python(data)
    elif get_origin(returnty) == list:
        return [load_data(item, get_args(returnty)[0]) for item in data]
    elif get_origin(returnty) == dict:
        return {load_data(k, get_args(returnty)[0]): load_data(v, get_args(returnty)[1]) for k, v in data.items()}
    elif get_origin(returnty) == tuple:
        return tuple(load_data(data, ty) for data, ty in zip(data, get_args(returnty)))
    elif get_origin(returnty) == types.UnionType:
        for arg in get_args(returnty):
            try:
                return load_data(data, arg)
            except Exception:
                pass
        raise ValueError(f"Unsupported value: {data} {returnty}")
    elif returnty is int:
        return int(data)
    elif returnty is float:
        return float(data)
    elif returnty is str:
        return str(data)
    elif returnty is bool:
        return bool(data)
    elif returnty is None:
        if data == '__none__':
            return None
        else:
            raise ValueError(f"Unsupported value for None: {data}")
    else:
        raise TypeError(f"Unsupported type: {returnty}")

def cache_fn(cache_dir: str):
    cache_dir = str(cache_dir)
    if not os.path.exists(cache_dir):
        os.makedirs(cache_dir)

    def decorator(func):
        def cached_func(*args, no_cache=False, **kwargs):
            sig = inspect.signature(func)
            bound = sig.bind(*args, **kwargs)
            bound.apply_defaults()

            input_dict = {}
            for k, v in bound.arguments.items():
                input_dict[k] = dump_data(v)

            input_hash = hash_dict_no_order(input_dict)
            cache_file = os.path.join(cache_dir, f"{func.__name__}{input_hash}.json")
            if os.path.exists(cache_file):
                with open(cache_file, "r") as f:
                    result = json.load(f)
                    with open(cache_file, "w") as f:
                        json.dump(result, f, ensure_ascii=False)
                    bound = sig.bind(**result["in"])
                    bound.apply_defaults()
                    assert dict(bound.arguments) == input_dict
                    result = load_data(result["out"], func.__annotations__["return"])
                    if no_cache:
                        os.remove(cache_file)
                    else:
                        return result


            result = func(*args, **kwargs)
            with open(cache_file, "w") as f:
                 json.dump({"in": input_dict, "out": dump_data(result)}, f, ensure_ascii=False)
            return result
        return cached_func
    return decorator
