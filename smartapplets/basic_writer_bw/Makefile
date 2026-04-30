CC = m68k-elf-gcc
LD = m68k-elf-ld

CFLAGS = -m68000 -mpcrel -fomit-frame-pointer -ffixed-a5 -ffixed-d7 -fshort-enums
CFLAGS += -Os -I"../betawise-sdk" -ffunction-sections -fdata-sections -ffreestanding
LDFLAGS = -static -T BasicWriterBW.lds --gc-sections
OBJ = BasicWriterBW.o syscall.o
TARGET = BasicWriterBW.elf

all: $(TARGET)

%.o: %.c
	$(CC) $(CFLAGS) -c $<

syscall.o: ../betawise-sdk/syscall.c
	$(CC) $(CFLAGS) -c $< -o $@

$(TARGET): $(OBJ) BasicWriterBW.lds
	$(LD) $(OBJ) $(LDFLAGS) -o $@

clean:
	rm -f BasicWriterBW.o syscall.o BasicWriterBW.elf
