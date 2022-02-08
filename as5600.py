#!/usr/bin/python3
import os
from cffi import FFI

ffi = FFI()
ffi.cdef("""
    typedef void* AS5600;

    AS5600 open_as5600_ffi(char* path);
    double get_angle_ffi(AS5600 encoder);
    void test_ffi(AS5600 encoder);
""")

C = ffi.dlopen(f"/home/{os.environ['USER']}/.local/lib/libas5600.so")

class AS5600:
    def __init__(self, path):
        self.ptr = C.open_as5600_ffi(path)

    def angle(self):
        return C.get_angle_ffi(self.ptr)

    def test(self):
        C.test_ffi(self.ptr)

if __name__ == "__main__":
    encoder = AS5600(b"/dev/i2c-2")
    encoder.test()  # this runs the read loop in rust at 200hz
    # while(True): print(encoder.angle())   # This runs the read loop in python as fast as possible 
