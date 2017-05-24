#ifndef CEXR_RUST_OSTREAM_H_
#define CEXR_RUST_OSTREAM_H_

#include "ImfIO.h"
#include "ImfInt64.h"

#include <cstddef>
#include <cstdint>

class RustOStream: public Imf::OStream {
public:
    RustOStream(
        void *writer,
        int (*write_ptr)(void *, const char *, int),
        uint64_t (*tellp_ptr)(void *),
        int (*seekp_ptr)(void *, uint64_t)
    )
        : OStream{"Rust StreamWriter"},
        writer{writer},
        write_ptr{write_ptr},
        tellp_ptr{tellp_ptr},
        seekp_ptr{seekp_ptr}
    {}

    virtual void write (const char c[/*n*/], int n);
    virtual uint64_t tellp ();
    virtual void seekp (uint64_t pos);

private:
    void *writer;
    int (*write_ptr)(void *, const char *, int);
    uint64_t (*tellp_ptr)(void *);
    int (*seekp_ptr)(void *, uint64_t);
};

#endif
