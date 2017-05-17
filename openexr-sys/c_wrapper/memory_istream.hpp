#ifndef CEXR_MEMORY_ISTREAM_H_
#define CEXR_MEMORY_ISTREAM_H_

#include "ImfIO.h"

#include <cstddef>

class MemoryIStream: public Imf::IStream {
public:
    MemoryIStream(const char *filename, char *data, std::size_t size)
        : IStream{filename}, data_{data}, position_{0}, size_{size} {}

    bool read(char *c, int n) override;
    IMATH_NAMESPACE::Int64 tellg() override;
    void seekg(IMATH_NAMESPACE::Int64 pos) override;
    bool isMemoryMapped() const override;
    char *readMemoryMapped(int n) override;

private:
    char *data_;
    std::size_t position_;
    std::size_t size_;
};

#endif
