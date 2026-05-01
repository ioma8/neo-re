CC = m68k-elf-gcc
LD = m68k-elf-ld

CFLAGS = -m68000 -mpcrel -fomit-frame-pointer -ffixed-a5 -ffixed-d7 -fshort-enums
CFLAGS += -O1 -fno-inline -fno-optimize-sibling-calls -I"../betawise-sdk" -ffunction-sections -fdata-sections -ffreestanding
LDFLAGS = -static -T WriteOrDieBW.lds --gc-sections
OBJ = WriteOrDieBW.o editor.o challenge.o ui.o storage.o syscall.o file_store.o
TARGET = WriteOrDieBW.elf

all: $(TARGET)

%.o: %.c
	$(CC) $(CFLAGS) -c $<

syscall.o: ../betawise-sdk/syscall.c
	$(CC) $(CFLAGS) -c $< -o $@

file_store.o: ../betawise-sdk/file_store.c
	$(CC) $(CFLAGS) -c $< -o $@

$(TARGET): $(OBJ) WriteOrDieBW.lds
	$(LD) $(OBJ) $(LDFLAGS) -o $@

clean:
	rm -f WriteOrDieBW.o editor.o challenge.o ui.o storage.o syscall.o file_store.o WriteOrDieBW.elf
