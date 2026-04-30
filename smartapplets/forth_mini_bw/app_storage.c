#include "app_storage.h"
#include "src/forth_util.h"

extern void SYS_A190(unsigned long handle, unsigned long offset, unsigned long mode);
extern void FileWriteBuffer(unsigned long handle, const void* data, unsigned long len, unsigned long flags);
extern void FileReadBuffer(unsigned long handle, void* data, unsigned long len, unsigned long flags);
extern void FileClose(void);
extern void SYS_A2BC(void);
extern void SYS_A2C0(void);
extern void SYS_A2DC(void);
extern unsigned long SYS_A2EC(void);
extern void SYS_A2FC(void);

enum { CURRENT_FILE = 0, MODE_READ = 1, MODE_WRITE = 2, IO_FLAGS = 1 };

static void ensure_workspace_ready(void) {
    SYS_A2DC();
    if(SYS_A2EC() == 0) {
        SYS_A2FC();
    }
}

int storage_load(char* buffer, size_t capacity) {
    if(capacity == 0) return 0;
    forth_memset(buffer, 0, capacity);
    return 1;
#if 0
    ensure_workspace_ready();
    SYS_A190(CURRENT_FILE, 0, MODE_READ);
    FileReadBuffer(CURRENT_FILE, buffer, (unsigned long)(capacity - 1), IO_FLAGS);
    FileClose();
    return 1;
#endif
}

int storage_save(const char* buffer) {
    (void)buffer;
    return 1;
#if 0
    ensure_workspace_ready();
    SYS_A190(CURRENT_FILE, 0, MODE_WRITE);
    FileWriteBuffer(CURRENT_FILE, buffer, (unsigned long)forth_strlen(buffer), IO_FLAGS);
    FileClose();
    SYS_A2BC();
    SYS_A2C0();
    return 1;
#endif
}
