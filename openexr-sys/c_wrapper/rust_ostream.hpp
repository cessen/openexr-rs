#ifndef CEXR_RUST_OSTREAM_H_
#define CEXR_RUST_OSTREAM_H_

#include "ImfIO.h"

#include <cstdint>

class RustOStream: public Imf::OStream {
public:
    RustOStream(
        void *writer,
        int (*write_ptr)(void *, const char *, int),
        int (*seekp_ptr)(void *, std::uint64_t)
    )
        : OStream{"Rust StreamWriter"},
        writer{writer},
        write_ptr{write_ptr},
        seekp_ptr{seekp_ptr},
        cursor_pos{0}
    {
        seekp_ptr(writer, 0);
    }

    virtual void write (const char c[/*n*/], int n);
    virtual std::uint64_t tellp ();
    virtual void seekp (std::uint64_t pos);

private:
    void *writer;
    int (*write_ptr)(void *, const char *, int);
    std::uint64_t (*tellp_ptr)(void *);
    int (*seekp_ptr)(void *, std::uint64_t);
    std::uint64_t cursor_pos;
};

#endif
