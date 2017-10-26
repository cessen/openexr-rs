#ifndef CEXR_RUST_OSTREAM_H_
#define CEXR_RUST_OSTREAM_H_

#include "ImfIO.h"
#include "ImfInt64.h"

#include <cstdint>

class RustOStream: public Imf::OStream {
public:
    RustOStream(
        void *writer,
        int (*write_ptr)(void *, const char *, int, int *err_out),
        int (*seekp_ptr)(void *, Imath::Int64, int *err_out)
    )
        : OStream{"Rust StreamWriter"},
        writer{writer},
        write_ptr{write_ptr},
        seekp_ptr{seekp_ptr},
        cursor_pos{0}
    {
        seekp(0);
    }

    virtual void write (const char c[/*n*/], int n);
    virtual Imath::Int64 tellp ();
    virtual void seekp (Imath::Int64 pos);

private:
    void *writer;
    int (*write_ptr)(void *, const char *, int, int *err_out);
    Imath::Int64 (*tellp_ptr)(void *);
    int (*seekp_ptr)(void *, Imath::Int64, int *err_out);
    Imath::Int64 cursor_pos;
};

#endif
