from importlib import import_module as _im
_core = _im(__name__ + '._core')      # -> qrmi._core  (Rust)

globals().update({k: v for k, v in _core.__dict__.items() if not k.startswith('_')})