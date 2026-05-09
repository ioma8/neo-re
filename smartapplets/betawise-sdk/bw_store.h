#ifndef BETAWISE_BW_STORE_H
#define BETAWISE_BW_STORE_H

#include <stddef.h>

typedef struct {
    char magic[4];
    size_t payload_size;
} BwSnapshotHeader;

int bw_store_load_snapshot(
    unsigned long file_handle,
    const char magic[4],
    void* payload,
    size_t payload_size);

int bw_store_save_snapshot(
    unsigned long file_handle,
    const char magic[4],
    const void* payload,
    size_t payload_size);

#endif
