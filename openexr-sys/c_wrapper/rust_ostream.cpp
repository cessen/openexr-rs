#include "rust_ostream.hpp"

#include <stdexcept>
#include <system_error>

using namespace IMATH_NAMESPACE;

void RustOStream::write(const char c[], int n) {
    int res = write_ptr(writer, c, n);
    if (res != 0) {
        throw std::system_error(res, std::system_category());
    }
    cursor_pos += n;
}

std::uint64_t RustOStream::tellp() {
    return cursor_pos;
}

void RustOStream::seekp(std::uint64_t pos) {
    int res = seekp_ptr(writer, pos);
    if (res != 0) {
        throw std::system_error(res, std::system_category());
    }
    cursor_pos = pos;
}