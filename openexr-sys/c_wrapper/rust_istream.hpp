#ifndef CEXR_RUST_ISTREAM_H_
#define CEXR_RUST_ISTREAM_H_

#include "ImfIO.h"

#include <cstddef>
#include <cstdint>

class RustIStream: public Imf::IStream {
public:
    RustIStream(
        void *reader,
        int (*read_ptr)(void *, char *, int, int *err_out),
        int (*seekg_ptr)(void *, std::uint64_t, int *err_out)
    )
        : IStream{"Rust reader"},
        reader{reader},
        read_ptr{read_ptr},
        seekg_ptr{seekg_ptr},
        cursor_pos{0}
    {
        seekg(0);
    }

    bool read(char c[/*n*/], int n);
    Imath::Int64 tellg();
    void seekg(Imath::Int64 pos);

private:
    void *reader;
    int (*read_ptr)(void *, char *, int, int *err_out);
    Imath::Int64 (*tellp_ptr)(void *);
    int (*seekg_ptr)(void *, std::uint64_t, int *err_out);
    Imath::Int64 cursor_pos;
};

#endif
