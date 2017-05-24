#include "rust_ostream.hpp"

#include <cstring>
#include <stdexcept>

using namespace IMATH_NAMESPACE;

void RustOStream::write(const char c[], int n) {
    int res = write_ptr(writer, c, n);
    if (res != 0) {
        throw std::runtime_error("error writing data");
    }
    cursor_pos += n;
}

uint64_t RustOStream::tellp() {
    return cursor_pos;
}

void RustOStream::seekp(uint64_t pos) {
    int res = seekp_ptr(writer, pos);
    if (res != 0) {
        throw std::runtime_error("error seeking in OStream");
    }
    cursor_pos = pos;
}