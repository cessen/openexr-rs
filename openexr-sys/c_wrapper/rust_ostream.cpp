#include "rust_ostream.hpp"

#include <stdexcept>
#include <system_error>

using namespace IMATH_NAMESPACE;

void RustOStream::write(const char c[], int n) {
    int err = 0;
    int res = write_ptr(writer, c, n, &err);
    if (res == 0) {
        // Success
        cursor_pos += n;
    } else if (res == 1) {
        // System error
        throw std::system_error(err, std::system_category());
    } else {
        // Some other kind of error
        throw std::runtime_error("error writing to output");
    }
}

std::uint64_t RustOStream::tellp() {
    return cursor_pos;
}

void RustOStream::seekp(std::uint64_t pos) {
    int err = 0;
    int res = seekp_ptr(writer, pos, &err);
    if (res == 0) {
        // Success
        cursor_pos = pos;
    } else if (res == 1) {
        // System error
        throw std::system_error(err, std::system_category());
    } else {
        // Some other kind of error
        throw std::runtime_error("error seeking in output");
    }
}