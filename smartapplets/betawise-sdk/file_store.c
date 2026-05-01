#include "file_store.h"
#include "os3k.h"

extern void SYS_A190(unsigned long handle, unsigned long offset, unsigned long mode);
extern void FileWriteBuffer(unsigned long handle, const void* data, unsigned long len, unsigned long flags);
extern void FileReadBuffer(unsigned long handle, void* data, unsigned long len, unsigned long flags);
extern void FileClose(void);

enum { MODE_READ = 1, MODE_WRITE = 2, IO_FLAGS = 1 };

typedef struct {
    AppletSnapshotHeader header;
    unsigned char payload[1];
} AppletSnapshotBuffer;

static int header_matches(const AppletSnapshotHeader* header, const char magic[4], size_t payload_size) {
    return header->magic[0] == magic[0] && header->magic[1] == magic[1] &&
           header->magic[2] == magic[2] && header->magic[3] == magic[3] &&
           header->payload_size == payload_size;
}

int applet_load_snapshot(
    unsigned long file_handle,
    const char magic[4],
    void* payload,
    size_t payload_size) {
    AppletSnapshotHeader header;
    memset(&header, 0, sizeof(header));
    SYS_A190(file_handle, 0, MODE_READ);
    FileReadBuffer(file_handle, &header, (unsigned long)sizeof(header), IO_FLAGS);
    if(!header_matches(&header, magic, payload_size)) {
        FileClose();
        return 0;
    }
    FileReadBuffer(file_handle, payload, (unsigned long)payload_size, IO_FLAGS);
    FileClose();
    return 1;
}

int applet_save_snapshot(
    unsigned long file_handle,
    const char magic[4],
    const void* payload,
    size_t payload_size) {
    AppletSnapshotHeader header;
    header.magic[0] = magic[0];
    header.magic[1] = magic[1];
    header.magic[2] = magic[2];
    header.magic[3] = magic[3];
    header.payload_size = payload_size;
    SYS_A190(file_handle, 0, MODE_WRITE);
    FileWriteBuffer(file_handle, &header, (unsigned long)sizeof(header), IO_FLAGS);
    FileWriteBuffer(file_handle, payload, (unsigned long)payload_size, IO_FLAGS);
    FileClose();
    return 1;
}
