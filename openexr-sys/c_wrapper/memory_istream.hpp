#ifndef CEXR_MEMORY_ISTREAM_H_
#define CEXR_MEMORY_ISTREAM_H_

#include "ImfIO.h"

#include <cstddef>

class MemoryIStream: public Imf::IStream {
public:
    MemoryIStream(const char *filename, char *data, std::size_t size)
        : IStream{filename}, data_{data}, position_{0}, size_{size} {}

    bool read(char *c, int n);
    IMATH_NAMESPACE::Int64 tellg();
    void seekg(IMATH_NAMESPACE::Int64 pos);
    bool isMemoryMapped() const;
    char *readMemoryMapped(int n);

private:
    char *data_;
    std::size_t position_;
    std::size_t size_;
};

#endif
