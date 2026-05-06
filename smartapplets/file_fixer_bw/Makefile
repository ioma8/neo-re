CC = m68k-elf-gcc
LD = m68k-elf-ld

CFLAGS = -m68000 -mpcrel -fomit-frame-pointer -ffixed-a5 -ffixed-d7 -fshort-enums
CFLAGS += -Os -I"../betawise-sdk" -ffunction-sections -fdata-sections -ffreestanding
LDFLAGS = -static -T FileFixer.lds --gc-sections
OBJ = FileFixer.o syscall.o
TARGET = FileFixer.elf

all: $(TARGET)

%.o: %.c
	$(CC) $(CFLAGS) -c $<

syscall.o: ../betawise-sdk/syscall.c
	$(CC) $(CFLAGS) -c $< -o $@

$(TARGET): $(OBJ) FileFixer.lds
	$(LD) $(OBJ) $(LDFLAGS) -o $@

clean:
	rm -f FileFixer.o syscall.o FileFixer.elf
