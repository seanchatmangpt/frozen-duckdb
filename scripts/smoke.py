# scripts/smoke.py
import os, ctypes, json
from ctypes import c_void_p, c_char_p, POINTER

libdir = os.environ.get("KCURA_LIB_DIR", "./target/release")
lib = ctypes.CDLL(os.path.join(libdir, "libkcura_ffi.so"))

class KcStatus(ctypes.Structure):
    _fields_ = [("code", ctypes.c_int32), ("msg", c_char_p)]

lib.kc_new.restype = c_void_p
lib.kc_free.argtypes = [c_void_p]
lib.kc_exec_sql.argtypes = [c_void_p, c_char_p, POINTER(c_char_p)]
lib.kc_exec_sql.restype = KcStatus
lib.kc_register_hook.argtypes = [c_void_p, c_char_p]
lib.kc_register_hook.restype = KcStatus
lib.kc_metrics_snapshot.argtypes = [c_void_p, POINTER(c_char_p)]
lib.kc_metrics_snapshot.restype = KcStatus
lib.kc_free_string.argtypes = [c_char_p]

def must(st: KcStatus):
    if st.code != 0:
        msg = ctypes.string_at(st.msg).decode()
        lib.kc_free_string(st.msg)
        raise RuntimeError(msg)

eng = lib.kc_new()
assert eng

# 1) SQL round-trip
out = c_char_p()
must(lib.kc_exec_sql(eng, b"CREATE TABLE t(x INT); INSERT INTO t VALUES (1),(2); SELECT count(*) AS cnt FROM t;", ctypes.byref(out)))
print("SQL:", ctypes.string_at(out).decode()); lib.kc_free_string(out)

# 2) Hook that blocks negatives
spec = b'{"id":"guard-no-negatives","kind":"guard","predicate_lang":"sql","predicate_sql":"SELECT 1 WHERE EXISTS(SELECT 1 FROM t WHERE x<0)","action_kind":"block"}'
must(lib.kc_register_hook(eng, spec))

# 3) Violating tx (expect error)
out2 = c_char_p()
st = lib.kc_exec_sql(eng, b"INSERT INTO t VALUES (-9);", ctypes.byref(out2))
if st.code == 0:
    print("UNEXPECTED:", ctypes.string_at(out2).decode()); lib.kc_free_string(out2)
else:
    err = ctypes.string_at(st.msg).decode(); lib.kc_free_string(st.msg)
    print("Hook blocked as expected:", err)

# 4) Metrics
m = c_char_p()
must(lib.kc_metrics_snapshot(eng, ctypes.byref(m)))
print("Metrics:", json.loads(ctypes.string_at(m).decode()))
lib.kc_free_string(m)

lib.kc_free(eng)
