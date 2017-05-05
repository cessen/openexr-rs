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

typedef struct CEXR_Box2i {
    CEXR_V2i min, max;
} CEXR_Box2i;

// From IlmImf/ImfPixelType.h
typedef enum CEXR_PixelType {
    UINT   = 0,
    HALF   = 1,
    FLOAT  = 2,
} CEXR_PixelType;


// Opaque types
typedef struct CEXR_InputFile CEXR_InputFile;
typedef struct CEXR_OutputFile CEXR_OutputFile;
typedef struct CEXR_Header CEXR_Header;
typedef struct CEXR_FrameBuffer CEXR_FrameBuffer;
typedef struct CEXR_IStream CEXR_IStream;


CEXR_IStream *CEXR_IStream_from_memory(const char *filename, char *data, size_t size);
void CEXR_IStream_delete(CEXR_IStream *stream);


int CEXR_InputFile_from_file(const char *path, int threads, CEXR_InputFile **out, const char **err_out);
int CEXR_InputFile_from_stream(CEXR_IStream *stream, int threads, CEXR_InputFile **out, const char **err_out);
void CEXR_InputFile_delete(CEXR_InputFile *file);
const CEXR_Header *CEXR_InputFile_header(CEXR_InputFile *file);
void CEXR_InputFile_set_framebuffer(CEXR_InputFile *file, CEXR_FrameBuffer *framebuffer);
int CEXR_InputFile_read_pixels(CEXR_InputFile *file, int scanline_1, int scanline_2, const char **err_out);


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

#ifdef __cplusplus
}
#endif

#endif
