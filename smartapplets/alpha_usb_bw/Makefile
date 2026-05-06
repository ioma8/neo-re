CC = m68k-elf-gcc
LD = m68k-elf-ld

CFLAGS = -m68000 -mpcrel -fomit-frame-pointer -ffixed-a5 -ffixed-d7 -fshort-enums
CFLAGS += -Os -I"../betawise-sdk" -ffunction-sections -fdata-sections -ffreestanding
LDFLAGS = -static -T AlphaUsb.lds --gc-sections
OBJ = AlphaUsb.o syscall.o
TARGET = AlphaUsb.elf

all: $(TARGET)

%.o: %.c
	$(CC) $(CFLAGS) -c $<

syscall.o: ../betawise-sdk/syscall.c
	$(CC) $(CFLAGS) -c $< -o $@

$(TARGET): $(OBJ) AlphaUsb.lds
	$(LD) $(OBJ) $(LDFLAGS) -o $@

clean:
	rm -f AlphaUsb.o syscall.o AlphaUsb.elf
