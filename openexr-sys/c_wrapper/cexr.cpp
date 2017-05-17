#include "cexr.h"

#include <cstdint>
#include <cstddef>
#include "OpenEXR/ImathVec.h"
#include "OpenEXR/ImathBox.h"
#include "OpenEXR/ImfPixelType.h"
#include "OpenEXR/ImfChannelList.h"
#include "OpenEXR/ImfHeader.h"
#include "OpenEXR/ImfFrameBuffer.h"
#include "OpenEXR/ImfOutputFile.h"
#include "OpenEXR/ImfInputFile.h"
#include "OpenEXR/Iex.h"

#include "memory_istream.hpp"

using namespace IMATH_NAMESPACE;
using namespace Imf;

static_assert(sizeof(CEXR_V2i) == sizeof(V2i), "V2i size is correct");
static_assert(sizeof(CEXR_Box2i) == sizeof(Box2i), "Box2i size is correct");

CEXR_IStream *CEXR_IStream_from_memory(const char *filename, char *data, size_t size) {
    return reinterpret_cast<CEXR_IStream *>(new MemoryIStream(filename, data, size));
}

void CEXR_IStream_delete(CEXR_IStream *stream) {
    delete reinterpret_cast<IStream *>(stream);
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

CEXR_ChannelListIter *CEXR_Header_channel_list_iter(const CEXR_Header *header) {
    CEXR_ChannelListIter *channel_iter = new CEXR_ChannelListIter();
    channel_iter->begin = reinterpret_cast<const Header *>(header)->channels().begin();
    channel_iter->end = reinterpret_cast<const Header *>(header)->channels().end();
    return channel_iter;
}

void CEXR_Header_delete(CEXR_Header *header) {
    delete reinterpret_cast<Header *>(header);
}

const CEXR_Box2i *CEXR_Header_display_window(const CEXR_Header *file) {
    return reinterpret_cast<const CEXR_Box2i *>(&reinterpret_cast<const Header *>(file)->displayWindow());
}
const CEXR_Box2i *CEXR_Header_data_window(const CEXR_Header *file) {
    return reinterpret_cast<const CEXR_Box2i *>(&reinterpret_cast<const Header *>(file)->dataWindow());
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


//----------------------------------------------------
// InputFile

int CEXR_InputFile_from_file(const char *path, int threads, CEXR_InputFile **out, const char **err_out) {
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

void CEXR_InputFile_set_framebuffer(CEXR_InputFile *file, CEXR_FrameBuffer *fb) {
    reinterpret_cast<InputFile *>(file)->setFrameBuffer(*reinterpret_cast<FrameBuffer *>(fb));
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

int CEXR_OutputFile_from_file(const char *path, const CEXR_Header *header, int threads, CEXR_OutputFile **out, const char **err_out) {
    try {
        *out = reinterpret_cast<CEXR_OutputFile *>(new OutputFile(path, *reinterpret_cast<const Header *>(header), threads));
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

void CEXR_OutputFile_set_framebuffer(CEXR_OutputFile *file, CEXR_FrameBuffer *fb) {
    reinterpret_cast<OutputFile *>(file)->setFrameBuffer(*reinterpret_cast<FrameBuffer *>(fb));
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