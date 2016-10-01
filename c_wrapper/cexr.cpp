#include <cstdint>
#include <cstddef>
#include "OpenEXR/ImfPixelType.h"
#include "OpenEXR/ImfHeader.h"
#include "OpenEXR/ImfFrameBuffer.h"
#include "OpenEXR/ImfOutputFile.h"
#include "OpenEXR/ImfInputFile.h"

extern "C" {
    // PixelType
    // This is a stand-in for an enum from the C++ library.
    // 0: u32
    // 1: f16
    // 2: f32
    typedef int CEXR_PixelType;

    // Channel
    // This isn't a wrapper per se, but an separate representation for
    // passing to/from Rust.
    struct CEXR_Channel {
        CEXR_PixelType type; // enum
        int xSampling;
        int ySampling;
        int pLinear; // bool
    };
};


//------------------------------------------------------------------------------
// Channel iterator
extern "C" {
    struct CEXR_ChannelIterator {
        void *channel_iterator
    };

    void CEXR_ChannelIterator_delete(
        CEXR_ChannelIterator *iterator);

    int CEXR_ChannelIterator_are_more(
        CEXR_ChannelIterator* iterator);

    const char[] CEXR_ChannelIterator_next(
        CEXR_ChannelIterator* iterator);
};


//------------------------------------------------------------------------------
// EXR header type.
extern "C" {
    struct CEXR_Header {
        void *header;
    };

    CEXR_Header CEXR_Header_new(
        int display_window_min_x,
        int display_window_min_y,
        int display_window_max_x,
        int display_window_max_y,
        int data_window_min_x,
        int data_window_min_y,
        int data_window_max_x,
        int data_window_max_y,
        float pixel_aspect_ratio,
        float screen_window_center_x,
        float screen_window_center_y,
        float screen_window_width,
        int line_order,
        int compression);

    void CEXR_Header_delete(
        CEXR_Header *header);

    void CEXR_Header_insert_channel(
        CEXR_Header *header,
        const char name[],
        const CEXR_Channel channel);

    int CEXR_Header_channel_exists(
        CEXR_Header *header,
        const char name[]);

    CEXR_Channel CEXR_Header_get_channel(
        CEXR_Header *header,
        const char name[]);

    CEXR_ChannelIterator CEXR_Header_new_channel_iterator(
        CEXR_Header *header);
};


//------------------------------------------------------------------------------
// FrameBuffer
extern "C" {
    struct CEXR_FrameBuffer {
        void *frame_buffer;
    };

    CEXR_FrameBuffer CEXR_FrameBuffer_new();

    void CEXR_FrameBuffer_delete(
        CEXR_FrameBuffer *frame_buffer);

    void CEXR_FrameBuffer_insert_slice(
        CEXR_FrameBuffer *frame_buffer,
        const char name[],
        char *base,
        size_t x_stride,
        size_t y_stride,
        int x_sampling,
        int y_sampling,
        double fill_value,
        int x_tile_coords, // bool
        int y_tile_coords // bool
        );
};


//------------------------------------------------------------------------------
// OutputFile
extern "C" {
    struct CEXR_OutputFile {
        void *output_file;
    };

    CEXR_OutputFile CEXR_OutputFile_new(
        const char[] file_name,
        const CEXR_Header *header,
        int num_threads);

    void CEXR_OutputFile_delete(
        CEXR_OutputFile *output_file);

    void CEXR_OutputFile_set_frame_buffer(
        CEXR_OutputFile* output_file,
        CEXR_FrameBuffer* frame_buffer);

    void CEXR_OutputFile_write_pixels(
        CEXR_OutputFile* output_file,
        int num_scan_lines);
};


//------------------------------------------------------------------------------
// InputFile
extern "C" {
    struct CEXR_InputFile {
        CEXR_Header header;
        void *input_file;
    };

    CEXR_InputFile CEXR_InputFile_new(
        const char file_name[],
        int num_threads);

    void CEXR_InputFile_delete(
        CEXR_InputFile *input_file);

    const CEXR_Header *CEXR_InputFile_header(
        const CEXR_InputFile *input_file);

    int CEXR_InputFile_version(
        const CEXR_InputFile *input_file);

    void CEXR_InputFile_set_frame_buffer(
        CEXR_InputFile* input_file,
        CEXR_FrameBuffer* frame_buffer);

    int CEXR_InputFile_is_complete(
        const CEXR_InputFile *input_file);

    void CEXR_InputFile_read_pixels(
        CEXR_InputFile *input_file,
        int scanline_1,
        int scanline_2);
};
