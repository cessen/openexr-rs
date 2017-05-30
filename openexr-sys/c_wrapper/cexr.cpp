#include "cexr.h"

#include <cstdint>
#include <cstddef>
#include "ImathVec.h"
#include "ImathBox.h"
#include "ImfPixelType.h"
#include "ImfChannelList.h"
#include "ImfHeader.h"
#include "ImfFrameBuffer.h"
#include "ImfOutputFile.h"
#include "ImfInputFile.h"
#include "Iex.h"

#include "memory_istream.hpp"
#include "rust_istream.hpp"
#include "rust_ostream.hpp"

using namespace IMATH_NAMESPACE;
using namespace Imf;

static_assert(sizeof(CEXR_V2i) == sizeof(V2i), "V2i size is correct");
static_assert(sizeof(CEXR_Box2i) == sizeof(Box2i), "Box2i size is correct");

int CEXR_IStream_from_reader(
    void *reader,
    int (*read_ptr)(void *, char *, int, int *err_out),
    int (*seekp_ptr)(void *, uint64_t, int *err_out),
    CEXR_IStream **out,
    const char **err_out
) {
    try {
        *out = reinterpret_cast<CEXR_IStream *>(new RustIStream(reader, read_ptr, seekp_ptr));
    } catch(const std::exception &e) {
        *err_out = e.what();
        return 1;
    }

    return 0;
}

CEXR_IStream *CEXR_IStream_from_memory(const char *filename, char *data, size_t size) {
    return reinterpret_cast<CEXR_IStream *>(new MemoryIStream(filename, data, size));
}

void CEXR_IStream_delete(CEXR_IStream *stream) {
    delete reinterpret_cast<IStream *>(stream);
}

int CEXR_OStream_from_writer(
    void *writer,
    int (*write_ptr)(void *, const char *, int, int *err_out),
    int (*seekp_ptr)(void *, uint64_t, int *err_out),
    CEXR_OStream **out,
    const char **err_out
) {
    try {
        *out = reinterpret_cast<CEXR_OStream *>(new RustOStream(writer, write_ptr, seekp_ptr));
    } catch(const std::exception &e) {
        *err_out = e.what();
        return 1;
    }

    return 0;
}

void CEXR_OStream_delete(CEXR_OStream *stream) {
    delete reinterpret_cast<OStream *>(stream);
}


//----------------------------------------------------
// ChannelListIter

struct CEXR_ChannelListIter {
    ChannelList::ConstIterator begin;
    ChannelList::ConstIterator end;
};

bool CEXR_ChannelListIter_next(CEXR_ChannelListIter *iter, const char **name, CEXR_Channel *channel) {
    if (iter->begin == iter->end) {
        return false;
    } else {
        *name = iter->begin.name();
        *channel = *reinterpret_cast<const CEXR_Channel *>(&(iter->begin.channel()));
        iter->begin++;
        return true;
    }
}

void CEXR_ChannelListIter_delete(CEXR_ChannelListIter *iter) {
    delete iter;
}


//----------------------------------------------------
// Header

CEXR_Header* CEXR_Header_new(const CEXR_Box2i *displayWindow,
                             const CEXR_Box2i *dataWindow,
                             float pixelAspectRatio,
                             const CEXR_V2f *screenWindowCenter,
                             float screenWindowWidth,
                             CEXR_LineOrder lineOrder,
                             CEXR_Compression compression) {
    Header *header = new Header(*reinterpret_cast<const IMATH_NAMESPACE::Box2i *>(displayWindow),
	                            *reinterpret_cast<const IMATH_NAMESPACE::Box2i *>(dataWindow),
	                            pixelAspectRatio,
	                            *reinterpret_cast<const IMATH_NAMESPACE::V2f *>(screenWindowCenter),
	                            screenWindowWidth,
	                            static_cast<LineOrder>(lineOrder),
                                static_cast<Compression>(compression));
    return reinterpret_cast<CEXR_Header *>(header);
}

void CEXR_Header_insert_channel(CEXR_Header *header, const char name[], const CEXR_Channel channel) {
    auto h = reinterpret_cast<Header*>(header);
    h->channels().insert(name, *reinterpret_cast<const Channel *>(&channel));
}

int CEXR_Header_get_channel(const CEXR_Header *header, const char name[], const CEXR_Channel **out) {
    auto h = reinterpret_cast<const Header*>(header);

    auto channel_ptr = reinterpret_cast<const CEXR_Channel *>(h->channels().findChannel(name));

    if (channel_ptr != 0) {
        *out = channel_ptr;
        return 0;
    } else {
        return 1;
    }
}

CEXR_ChannelListIter *CEXR_Header_channel_list_iter(const CEXR_Header *header) {
    CEXR_ChannelListIter *channel_iter = new CEXR_ChannelListIter();
    channel_iter->begin = reinterpret_cast<const Header *>(header)->channels().begin();
    channel_iter->end = reinterpret_cast<const Header *>(header)->channels().end();
    return channel_iter;
}

void CEXR_Header_delete(CEXR_Header *header) {
    delete reinterpret_cast<Header *>(header);
}

const CEXR_Box2i *CEXR_Header_display_window(const CEXR_Header *header) {
    return reinterpret_cast<const CEXR_Box2i *>(&reinterpret_cast<const Header *>(header)->displayWindow());
}
const CEXR_Box2i *CEXR_Header_data_window(const CEXR_Header *header) {
    return reinterpret_cast<const CEXR_Box2i *>(&reinterpret_cast<const Header *>(header)->dataWindow());
}

void CEXR_Header_set_display_window(CEXR_Header *header, CEXR_Box2i window) {
    *reinterpret_cast<CEXR_Box2i *>(&reinterpret_cast<Header *>(header)->displayWindow()) = window;
}

void CEXR_Header_set_data_window(CEXR_Header *header, CEXR_Box2i window) {
    *reinterpret_cast<CEXR_Box2i *>(&reinterpret_cast<Header *>(header)->dataWindow()) = window;
}

void CEXR_Header_set_pixel_aspect_ratio(CEXR_Header *header, float aspect_ratio) {
    reinterpret_cast<Header *>(header)->pixelAspectRatio() = aspect_ratio;
}

void CEXR_Header_set_screen_window_center(CEXR_Header *header, CEXR_V2f center) {
    *reinterpret_cast<CEXR_V2f *>(&reinterpret_cast<Header *>(header)->screenWindowCenter()) = center;
}

void CEXR_Header_set_screen_window_width(CEXR_Header *header, float width) {
    reinterpret_cast<Header *>(header)->screenWindowWidth() = width;
}

void CEXR_Header_set_line_order(CEXR_Header *header, CEXR_LineOrder line_order) {
    *reinterpret_cast<CEXR_LineOrder *>(&reinterpret_cast<Header *>(header)->lineOrder()) = line_order;
}

void CEXR_Header_set_compression(CEXR_Header *header, CEXR_Compression compression) {
    *reinterpret_cast<CEXR_Compression *>(&reinterpret_cast<Header *>(header)->compression()) = compression;
}


//----------------------------------------------------
// FrameBuffer

CEXR_FrameBuffer *CEXR_FrameBuffer_new() {
    return reinterpret_cast<CEXR_FrameBuffer *>(new FrameBuffer);
}

void CEXR_FrameBuffer_delete(CEXR_FrameBuffer *fb) {
    delete reinterpret_cast<FrameBuffer *>(fb);
}

void CEXR_FrameBuffer_insert(CEXR_FrameBuffer *fb,
                             const char *name,
                             CEXR_PixelType type,
                             char * base,
                             size_t xStride,
                             size_t yStride,
                             int xSampling,
                             int ySampling,
                             double fillValue,
                             int xTileCoords,
                             int yTileCoords) {
    reinterpret_cast<FrameBuffer *>(fb)->insert(name, Slice(static_cast<Imf::PixelType>(type), base, xStride, yStride, xSampling, ySampling, fillValue, xTileCoords, yTileCoords));
}

int CEXR_FrameBuffer_get_channel(const CEXR_FrameBuffer *frame_buffer, const char name[], CEXR_Channel *out) {
    auto fb = reinterpret_cast<const FrameBuffer*>(frame_buffer);

    auto slice_ptr = fb->findSlice(name);

    if (slice_ptr != 0) {
        *out = CEXR_Channel {
            *reinterpret_cast<const CEXR_PixelType *>(&(slice_ptr->type)),
            slice_ptr->xSampling,
            slice_ptr->ySampling,
            false // Bogus value, but this function is only used internally anyway
        };
        return 0;
    } else {
        return 1;
    }
}

// Creates a copy of the framebuffer, but with all base pointers offset by
// `offset` scanlines.
//
// For example, if you specify an offset of 3, then if you access scanline 3
// of the new framebuffer it will be the same as accessing scanline 0 of the
// old one.
CEXR_FrameBuffer *CEXR_FrameBuffer_copy_and_offset_scanlines(const CEXR_FrameBuffer *frame_buffer, unsigned int offset) {
    auto fb = reinterpret_cast<const FrameBuffer *>(frame_buffer);

    auto new_fb = new FrameBuffer();

    // Copy all of the slices to the new frame buffer while offsetting their
    // base pointers appropriately.
    for (auto itr = fb->begin(); itr != fb->end(); itr++) {
        Slice slice = itr.slice();

        auto tmp = (size_t)slice.base;
        tmp -= slice.yStride * (offset / slice.ySampling);
        slice.base = (char *)tmp;

        new_fb->insert(itr.name(), slice);
    }

    return reinterpret_cast<CEXR_FrameBuffer *>(new_fb);
}


//----------------------------------------------------
// InputFile

int CEXR_InputFile_from_file_path(const char *path, int threads, CEXR_InputFile **out, const char **err_out) {
    try {
        *out = reinterpret_cast<CEXR_InputFile *>(new InputFile(path, threads));
    } catch(const std::exception &e) {
        *err_out = e.what();
        return 1;
    }

    return 0;
}

int CEXR_InputFile_from_stream(CEXR_IStream *stream, int threads, CEXR_InputFile **out, const char **err_out) {
    try {
        *out = reinterpret_cast<CEXR_InputFile *>(new InputFile(*reinterpret_cast<IStream *>(stream), threads));
    } catch(const std::exception &e) {
        *err_out = e.what();
        return 1;
    }

    return 0;
}

void CEXR_InputFile_delete(CEXR_InputFile *file) {
    delete reinterpret_cast<InputFile *>(file);
}

const CEXR_Header *CEXR_InputFile_header(CEXR_InputFile *file) {
    return reinterpret_cast<const CEXR_Header *>(&reinterpret_cast<InputFile *>(file)->header());
}

int CEXR_InputFile_set_framebuffer(CEXR_InputFile *file, CEXR_FrameBuffer *fb, const char **err_out) {
    try {
        reinterpret_cast<InputFile *>(file)->setFrameBuffer(*reinterpret_cast<FrameBuffer *>(fb));
    } catch(const std::exception &e) {
        *err_out = e.what();
        return 1;
    }

    return 0;
}

int CEXR_InputFile_read_pixels(CEXR_InputFile *file, int scanline_1, int scanline_2, const char **err_out) {
    try {
        reinterpret_cast<InputFile *>(file)->readPixels(scanline_1, scanline_2);
    } catch(const std::exception &e) {
        *err_out = e.what();
        return 1;
    }
    return 0;
}


//----------------------------------------------------
// OutputFile

int CEXR_OutputFile_from_stream(CEXR_OStream *stream, const CEXR_Header *header, int threads, CEXR_OutputFile **out, const char **err_out) {
    try {
        *out = reinterpret_cast<CEXR_OutputFile *>(new OutputFile(*reinterpret_cast<OStream *>(stream), *reinterpret_cast<const Header *>(header), threads));
    } catch(const std::exception &e) {
        *err_out = e.what();
        return 1;
    }

    return 0;
}

void CEXR_OutputFile_delete(CEXR_OutputFile *file) {
    delete reinterpret_cast<OutputFile *>(file);
}

const CEXR_Header *CEXR_OutputFile_header(CEXR_OutputFile *file) {
    return reinterpret_cast<const CEXR_Header *>(&reinterpret_cast<OutputFile *>(file)->header());
}

int CEXR_OutputFile_set_framebuffer(CEXR_OutputFile *file, const CEXR_FrameBuffer *fb, const char **err_out) {
    try {
        reinterpret_cast<OutputFile *>(file)->setFrameBuffer(*reinterpret_cast<const FrameBuffer *>(fb));
    } catch(const std::exception &e) {
        *err_out = e.what();
        return 1;
    }

    return 0;
}

int CEXR_OutputFile_write_pixels(CEXR_OutputFile *file, int num_scanlines, const char **err_out) {
    try {
        reinterpret_cast<OutputFile *>(file)->writePixels(num_scanlines);
    } catch(const std::exception &e) {
        *err_out = e.what();
        return 1;
    }
    return 0;
}
