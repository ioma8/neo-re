CC = m68k-elf-gcc
LD = m68k-elf-ld

CFLAGS = -m68000 -mpcrel -fomit-frame-pointer -ffixed-a5 -ffixed-d7 -fshort-enums
CFLAGS += -O1 -fno-inline -fno-optimize-sibling-calls -I"../betawise-sdk" -ffunction-sections -fdata-sections -ffreestanding
LDFLAGS = -static -T DoomBW.lds --gc-sections
OBJ = DoomBW.o game.o render.o syscall.o
TARGET = DoomBW.elf

all: $(TARGET)

%.o: %.c
	$(CC) $(CFLAGS) -c $<

syscall.o: ../betawise-sdk/syscall.c
	$(CC) $(CFLAGS) -c $< -o $@

$(TARGET): $(OBJ) DoomBW.lds
	$(LD) $(OBJ) $(LDFLAGS) -o $@

clean:
	rm -f DoomBW.o game.o render.o syscall.o DoomBW.elf
