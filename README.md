# OpenEXR-rs

Rust bindings for [OpenEXR](http://www.openexr.com).

## Overview

OpenEXR is a bitmap image file format that can store high dynamic range images
along with other arbitrary per-pixel data.  It is used heavily in the VFX and
3D animation industries.

The goal of this library is to wrap and support all features of OpenEXR 2.x.
Convenient and safe API's will be provided for most functionality.  However,
to provide the full flexibility of the underlying C++ library, there may be
a few unsafe API's as well that expose more nitty-gritty.

## Using

You will need builds of OpenEXR and zlib available.  You can specify the
prefixes the libraries are installed to with the ILMBASE_DIR, OPENEXR_DIR, and
ZLIB_DIR environment variables.  Depending on how your OpenEXR was built, you
may also need to set OPENEXR_LIB_SUFFIX to a value such as "2_2".  If an _DIR
variable is unset, pkgconfig will be used to try to find the corresponding
library automatically.

## Status

This library has been tested on Linux and Windows.  Basic I/O is supported,
including reading from memory.

## TODO

- [x] Wrap basic scanline output.
- [x] Wrap basic scanline input.
- [x] Figure out a good way to support Half floats.
- [ ] Make simple convenience functions for basic RGB/RGBA input and output.
- [ ] Wrap basic tiled output.
- [ ] Wrap basic tiled input.
- [ ] Handle different tiled modes (e.g. MIP maps and RIP maps).
- [ ] Wrap deep data input/output.
- [ ] Wrap multi-part file input/output.
- [ ] Wrap custom attributes.
- [ ] Handle exceptions at the API boundary (safety!).
- [ ] Take stock after this is all done and figure a out better API for
      everything.
- [ ] For the sake of opaque pointers to the C++ classes, there is a lot of
      seemingly unnecessary heap allocation.  Can we move more of this to the
      stack?
- [ ] Make build system more robust to various platforms and configurations.

## License

OpenEXR-rs is distributed under the terms of the MIT license (see LICENSE for
details).  The code for OpenEXR itself is distributed under the terms of a
modified BSD license (see http://www.openexr.com/license.html for details).
zlib is distributed under the terms of the zlib license (see
http://zlib.net/zlib_license.html for details).
