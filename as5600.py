#!/usr/bin/python3
from cffi import FFI

ffi = FFI()
ffi.cdef("""
    typedef void* Context;

    Context context(char* path);
    double angle(Context ctx);
    void test(Context ctx);
""")

C = ffi.dlopen("/home/tag/.local/lib/libas5600.so")

def run():
    ctx = C.get_context(b"/dev/i2c-2")
    print(C.angle(ctx))

class AS5600:
    def __init__(self, path):
        self.ctx = C.context(path)

    def angle(self):
        return C.angle(self.ctx)

    def test(self):
        C.test(self.ctx)

if __name__ == "__main__":
    encoder = AS5600(b"/dev/i2c-2")
    encoder.test()
    # while(True): print(encoder.angle())
