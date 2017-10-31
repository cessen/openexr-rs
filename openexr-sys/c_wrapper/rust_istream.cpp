#include "rust_istream.hpp"

#include <stdexcept>
#include <system_error>

using namespace IMATH_NAMESPACE;

bool RustIStream::read(char c[/*n*/], int n) {
    int err = 0;
    int res = read_ptr(reader, c, n, &err);
    if (res == 0) {
        // Success
        cursor_pos += n;
    } else if (res == 1) {
        // System error
        throw std::system_error(err, std::system_category());
    } else {
        // Some other kind of error
        throw std::runtime_error("error reading from input");
    }
}

Imath::Int64 RustIStream::tellg() {
    return cursor_pos;
}

void RustIStream::seekg(Imath::Int64 pos) {
    int err = 0;
    int res = seekg_ptr(reader, pos, &err);
    if (res == 0) {
        // Success
        cursor_pos = pos;
    } else if (res == 1) {
        // System error
        throw std::system_error(err, std::system_category());
    } else {
        // Some other kind of error
        throw std::runtime_error("error seeking in input");
    }
}
