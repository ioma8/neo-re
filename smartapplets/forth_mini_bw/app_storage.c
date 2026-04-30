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

enum { CURRENT_FILE = 1, MODE_READ = 1, MODE_WRITE = 2, IO_FLAGS = 1 };

typedef struct {
    char magic[4];
    ForthMachine machine;
} ForthSnapshot;

static void ensure_workspace_ready(void) {
    SYS_A2DC();
    if(SYS_A2EC() == 0) {
        SYS_A2FC();
    }
}

int storage_load_machine(ForthMachine* machine) {
    ForthSnapshot snapshot;
    forth_memset(&snapshot, 0, sizeof(snapshot));
    ensure_workspace_ready();
    SYS_A190(CURRENT_FILE, 0, MODE_READ);
    FileReadBuffer(CURRENT_FILE, &snapshot, (unsigned long)sizeof(snapshot), IO_FLAGS);
    FileClose();
    if(snapshot.magic[0] != 'F' || snapshot.magic[1] != 'M' || snapshot.magic[2] != 'N' || snapshot.magic[3] != '1') {
        return 0;
    }
    forth_memcpy(machine, &snapshot.machine, sizeof(*machine));
    return 1;
}

int storage_save_machine(const ForthMachine* machine) {
    ForthSnapshot snapshot;
    snapshot.magic[0] = 'F';
    snapshot.magic[1] = 'M';
    snapshot.magic[2] = 'N';
    snapshot.magic[3] = '1';
    forth_memcpy(&snapshot.machine, machine, sizeof(snapshot.machine));
    ensure_workspace_ready();
    SYS_A190(CURRENT_FILE, 0, MODE_WRITE);
    FileWriteBuffer(CURRENT_FILE, &snapshot, (unsigned long)sizeof(snapshot), IO_FLAGS);
    FileClose();
    return 1;
}
