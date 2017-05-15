#include "memory_istream.hpp"

#include <cstring>
#include <stdexcept>

using namespace IMATH_NAMESPACE;

bool MemoryIStream::read(char *c, int n) {
    if(position_ + n <= size_) {
        memcpy(c, static_cast<void *>(data_ + position_), n);
        position_ += n;
    } else {
        throw std::runtime_error("unexpected EOF");
    }

    return position_ != size_;
}

Int64 MemoryIStream::tellg() {
    return position_;
}

void MemoryIStream::seekg(Int64 pos) {
    position_ = pos;
}

bool MemoryIStream::isMemoryMapped() const {
    return true;
}

char *MemoryIStream::readMemoryMapped(int n) {
    if(position_ + n > size_) {
        throw std::runtime_error("unexpected EOF");
    }

    std::size_t start = position_;
    position_ += n;
    return data_ + start;
}
