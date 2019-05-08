#ifndef CEXR_H_
#define CEXR_H_

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Transparent types
/**
 * A 2d integer vector.
 *
 * Used in various parts of OpenEXR's APIs.
 */
typedef struct CEXR_V2i {
    int x, y;
} CEXR_V2i;

/**
 * A 2d floating point vector.
 *
 * Used in various parts of OpenEXR's APIs.
 */
typedef struct CEXR_V2f {
    float x, y;
} CEXR_V2f;

/**
 * A 2d integer bounding box.
 *
 * Used in various parts of OpenEXR's APIs.
 */
typedef struct CEXR_Box2i {
    CEXR_V2i min, max;
} CEXR_Box2i;

// From IlmImf/ImfPixelType.h
/**
 * Describes the datatype of an image channel.
 *
 * * `UINT`: 32-bit unsigned integer.
 * * `HALF`: 16-bit floating point (conforming to IEEE 754).
 * * `FLOAT`: 32-bit floating point (conforming to IEEE 754)
 */
typedef enum CEXR_PixelType {
    UINT   = 0,
    HALF   = 1,
    FLOAT  = 2,
} CEXR_PixelType;

// from IlmImf/ImfLineOrder.h
/**
 * Defines the line order of a scanline image.
 *
 * For scanline images, only `INCREASING_Y` and `DECREASING_Y` are valid
 * values:
 *
 * * `INCREASING_Y`: scanline 0 is the first scanline in the file, and
 *   scanlines are written and read in that order.
 *
 * * `DECREASING_Y`: scanline 0 is the last scanline in the file, and
 *   scanlines are written and read in that order.
 *
 * In both cases, scanlines are written to and read from files in the order
 * they are stored on disk, and any `FrameBuffer` you pass is interpretted
 * that way as well.
 *
 * For tiled images, all values are valid, but they have different meanings:
 *
 * * `INCREASING_Y`: the tiles are stored in a particular order.  See
 *   OpenEXR's
 *   [ImfTiledOutputFile.h]
 *   (https://github.com/openexr/openexr/blob/develop/OpenEXR/IlmImf/ImfTiledOutputFile.h)
 *   header for specifics.
 * 
 * * `DECREASING_Y`: the tiles are stored in a different particular order.
 *   See OpenEXR's
 *   [ImfTiledOutputFile.h]
 *   (https://github.com/openexr/openexr/blob/develop/OpenEXR/IlmImf/ImfTiledOutputFile.h)
 *   header for specifics.
 * 
 * * `RANDOM_Y`: the tiles are stored in the order written.
 *
 * For tiled files, `RANDOM_Y` is probably a good choice, as it gives you
 * control over the tile layout and doesn't require the OpenEXR library to
 * do any buffering.
 */
typedef enum CEXR_LineOrder {
    INCREASING_Y = 0,
    DECREASING_Y = 1,
    RANDOM_Y = 2,
} CEXR_LineOrder;

// from IlmImf/ImfCompression.h
/**
 * Compression mode of an OpenEXR file.
 *
 * These modes are lossless:
 * 
 * * `NO_COMPRESSION`
 * * `RLE_COMPRESSION`
 * * `ZIPS_COMPRESSION`
 * * `ZIP_COMPRESSION`
 * * `PIZ_COMPRESSION`
 *
 * These modes are lossy:
 *
 * * `PXR24_COMPRESSION`
 * * `B44_COMPRESSION`
 * * `B44A_COMPRESSION`
 * * `DWAA_COMPRESSION`
 * * `DWAB_COMPRESSION`
 *
 * And `PXR24_COMPRESSION` is only lossy for 32-bit floating point channels,
 * which it converts to 24-bit floating point.
 *
 * See OpenEXR's documentation and header files for more details on the
 * compression modes.
 */
typedef enum CEXR_Compression {
    NO_COMPRESSION  = 0,
    RLE_COMPRESSION = 1,
    ZIPS_COMPRESSION = 2,
    ZIP_COMPRESSION = 3,
    PIZ_COMPRESSION = 4,
    PXR24_COMPRESSION = 5,
    B44_COMPRESSION = 6,
    B44A_COMPRESSION = 7,
    DWAA_COMPRESSION = 8,
    DWAB_COMPRESSION = 9,
} CEXR_Compression;

// IlmImf/ImfChannelList.h
// Changed element names slightly to adhere to Rust naming conventions.
/**
 * Describes an image channel.
 */
typedef struct CEXR_Channel {
    CEXR_PixelType pixel_type;
    int x_sampling;
    int y_sampling;
    bool p_linear;
} CEXR_Channel;


// Opaque types
typedef struct CEXR_InputFile CEXR_InputFile;
typedef struct CEXR_OutputFile CEXR_OutputFile;
typedef struct CEXR_Header CEXR_Header;
typedef struct CEXR_FrameBuffer CEXR_FrameBuffer;
typedef struct CEXR_IStream CEXR_IStream;
typedef struct CEXR_OStream CEXR_OStream;
typedef struct CEXR_ChannelListIter CEXR_ChannelListIter;


int CEXR_IStream_from_reader(
    void *reader,
    int (*read_ptr)(void *, char *, int, int *err_out),
    int (*seekp_ptr)(void *, uint64_t, int *err_out),
    CEXR_IStream **out,
    const char **err_out
);
CEXR_IStream *CEXR_IStream_from_memory(const char *filename, char *data, size_t size);
void CEXR_IStream_delete(CEXR_IStream *stream);

int CEXR_OStream_from_writer(
    void *writer,
    int (*write_ptr)(void *, const char *, int, int *err_out),
    int (*seekp_ptr)(void *, uint64_t, int *err_out),
    CEXR_OStream **out,
    const char **err_out
);
void CEXR_OStream_delete(CEXR_OStream *stream);

bool CEXR_ChannelListIter_next(CEXR_ChannelListIter *iter, const char **name, CEXR_Channel *channel);
void CEXR_ChannelListIter_delete(CEXR_ChannelListIter *iter);

CEXR_Header *CEXR_Header_new(const CEXR_Box2i *displayWindow,
                             const CEXR_Box2i *dataWindow,
                             float pixelAspectRatio,
                             const CEXR_V2f *screenWindowCenter,
                             float screenWindowWidth,
                             CEXR_LineOrder lineOrder,
                             CEXR_Compression compression);
void CEXR_Header_delete(CEXR_Header *header);
void CEXR_Header_insert_channel(CEXR_Header *header, const char name[], const CEXR_Channel channel);
const CEXR_Channel *CEXR_Header_get_channel(const CEXR_Header *header, const char name[]);
CEXR_ChannelListIter *CEXR_Header_channel_list_iter(const CEXR_Header *header);
const CEXR_Box2i *CEXR_Header_display_window(const CEXR_Header *header);
const CEXR_Box2i *CEXR_Header_data_window(const CEXR_Header *header);
void CEXR_Header_set_display_window(CEXR_Header *header, CEXR_Box2i window);
void CEXR_Header_set_data_window(CEXR_Header *header, CEXR_Box2i window);
void CEXR_Header_set_pixel_aspect_ratio(CEXR_Header *header, float aspect_ratio);
void CEXR_Header_set_screen_window_center(CEXR_Header *header, CEXR_V2f center);
void CEXR_Header_set_screen_window_width(CEXR_Header *header, float width);
void CEXR_Header_set_line_order(CEXR_Header *header, CEXR_LineOrder line_order);
void CEXR_Header_set_compression(CEXR_Header *header, CEXR_Compression compression);
bool CEXR_Header_has_envmap(const CEXR_Header *header);
int CEXR_Header_envmap(const CEXR_Header *header);
void CEXR_Header_set_envmap(CEXR_Header *header, int envmap);
void CEXR_Header_erase_attribute(CEXR_Header *header, const char *attribute);


CEXR_FrameBuffer *CEXR_FrameBuffer_new();
void CEXR_FrameBuffer_delete(CEXR_FrameBuffer *framebuffer);
void CEXR_FrameBuffer_insert(CEXR_FrameBuffer *framebuffer,
                             const char *name,
                             CEXR_PixelType type,
                             char * base,
                             size_t xStride,
                             size_t yStride,
                             int xSampling,
                             int ySampling,
                             double fillValue,
                             int xTileCoords,
                             int yTileCoords);
int CEXR_FrameBuffer_get_channel(const CEXR_FrameBuffer *frame_buffer, const char name[], CEXR_Channel *out);
CEXR_FrameBuffer *CEXR_FrameBuffer_copy_and_offset_scanlines(const CEXR_FrameBuffer *frame_buffer, unsigned int offset);

int CEXR_InputFile_from_file_path(const char *path, int threads, CEXR_InputFile **out, const char **err_out);
int CEXR_InputFile_from_stream(CEXR_IStream *stream, int threads, CEXR_InputFile **out, const char **err_out);
void CEXR_InputFile_delete(CEXR_InputFile *file);
const CEXR_Header *CEXR_InputFile_header(CEXR_InputFile *file);
int CEXR_InputFile_set_framebuffer(CEXR_InputFile *file, CEXR_FrameBuffer *framebuffer, const char **err_out);
int CEXR_InputFile_read_pixels(CEXR_InputFile *file, int scanline_1, int scanline_2, const char **err_out);

int CEXR_OutputFile_from_stream(CEXR_OStream *stream, const CEXR_Header *header, int threads, CEXR_OutputFile **out, const char **err_out);
void CEXR_OutputFile_delete(CEXR_OutputFile *file);
const CEXR_Header *CEXR_OutputFile_header(CEXR_OutputFile *file);
int CEXR_OutputFile_set_framebuffer(CEXR_OutputFile *file, const CEXR_FrameBuffer *framebuffer, const char **err_out);
int CEXR_OutputFile_write_pixels(CEXR_OutputFile *file, int num_scanlines, const char **err_out);

int CEXR_set_global_thread_count(int thread_count);

#ifdef __cplusplus
}
#endif

#endif
