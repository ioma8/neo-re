CC = m68k-elf-gcc
LD = m68k-elf-ld

CFLAGS = -m68000 -mpcrel -fomit-frame-pointer -ffixed-a5 -ffixed-d7 -fshort-enums
CFLAGS += -O1 -fno-inline -fno-optimize-sibling-calls -I"../betawise-sdk" -ffunction-sections -fdata-sections -ffreestanding
LDFLAGS = -static -T ForthMiniBW.lds --gc-sections
OBJ = ForthMiniBW.o app_ui.o app_storage.o syscall.o forth_core.o forth_dict.o forth_vm.o forth_compile.o forth_util.o
TARGET = ForthMiniBW.elf

all: $(TARGET)

%.o: %.c
	$(CC) $(CFLAGS) -c $<

syscall.o: ../betawise-sdk/syscall.c
	$(CC) $(CFLAGS) -c $< -o $@

forth_core.o: src/forth_core.c
	$(CC) $(CFLAGS) -c $< -o $@

forth_dict.o: src/forth_dict.c
	$(CC) $(CFLAGS) -c $< -o $@

forth_vm.o: src/forth_vm.c
	$(CC) $(CFLAGS) -c $< -o $@

forth_compile.o: src/forth_compile.c
	$(CC) $(CFLAGS) -c $< -o $@

forth_util.o: src/forth_util.c
	$(CC) $(CFLAGS) -c $< -o $@

$(TARGET): $(OBJ) ForthMiniBW.lds
	$(LD) $(OBJ) $(LDFLAGS) -o $@

clean:
	rm -f ForthMiniBW.o app_ui.o app_storage.o syscall.o forth_core.o forth_dict.o forth_vm.o forth_compile.o forth_util.o ForthMiniBW.elf
