#ifndef CEXR_H_
#define CEXR_H_

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Transparent types
typedef struct CEXR_V2i {
    int x, y;
} CEXR_V2i;

typedef struct CEXR_V2f {
    float x, y;
} CEXR_V2f;

typedef struct CEXR_Box2i {
    CEXR_V2i min, max;
} CEXR_Box2i;

// From IlmImf/ImfPixelType.h
typedef enum CEXR_PixelType {
    UINT   = 0,
    HALF   = 1,
    FLOAT  = 2,
} CEXR_PixelType;

// from IlmImf/ImfLineOrder.h
typedef enum CEXR_LineOrder {
    INCREASING_Y = 0,
    DECREASING_Y = 1,
    RANDOM_Y = 2,
} CEXR_LineOrder;

// from IlmImf/ImfCompression.h
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


// Opaque types
typedef struct CEXR_InputFile CEXR_InputFile;
typedef struct CEXR_OutputFile CEXR_OutputFile;
typedef struct CEXR_Header CEXR_Header;
typedef struct CEXR_FrameBuffer CEXR_FrameBuffer;
typedef struct CEXR_IStream CEXR_IStream;


CEXR_IStream *CEXR_IStream_from_memory(const char *filename, char *data, size_t size);
void CEXR_IStream_delete(CEXR_IStream *stream);
CEXR_Header *CEXR_Header_new(const CEXR_Box2i *displayWindow,
                             const CEXR_Box2i *dataWindow,
                             float pixelAspectRatio,
                             const CEXR_V2f *screenWindowCenter,
                             float screenWindowWidth,
                             CEXR_LineOrder lineOrder,
                             CEXR_Compression compression);
void CEXR_Header_delete(CEXR_Header *header);
const CEXR_Box2i *CEXR_Header_display_window(const CEXR_Header *header);
const CEXR_Box2i *CEXR_Header_data_window(const CEXR_Header *header);

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

int CEXR_InputFile_from_file(const char *path, int threads, CEXR_InputFile **out, const char **err_out);
int CEXR_InputFile_from_stream(CEXR_IStream *stream, int threads, CEXR_InputFile **out, const char **err_out);
void CEXR_InputFile_delete(CEXR_InputFile *file);
const CEXR_Header *CEXR_InputFile_header(CEXR_InputFile *file);
void CEXR_InputFile_set_framebuffer(CEXR_InputFile *file, CEXR_FrameBuffer *framebuffer);
int CEXR_InputFile_read_pixels(CEXR_InputFile *file, int scanline_1, int scanline_2, const char **err_out);

int CEXR_OutputFile_from_file(const char *path, const CEXR_Header *header, int threads, CEXR_OutputFile **out, const char **err_out);
void CEXR_OutputFile_delete(CEXR_OutputFile *file);
const CEXR_Header *CEXR_OutputFile_header(CEXR_OutputFile *file);
void CEXR_OutputFile_set_framebuffer(CEXR_OutputFile *file, CEXR_FrameBuffer *framebuffer);
int CEXR_OutputFile_write_pixels(CEXR_OutputFile *file, int num_scanlines, const char **err_out);


#ifdef __cplusplus
}
#endif

#endif
