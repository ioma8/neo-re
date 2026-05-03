CC = m68k-elf-gcc
LD = m68k-elf-ld

CFLAGS = -m68000 -mpcrel -fomit-frame-pointer -ffixed-a5 -ffixed-d7 -fshort-enums
CFLAGS += -O1 -fno-inline -fno-optimize-sibling-calls -I"../betawise-sdk" -ffunction-sections -fdata-sections -ffreestanding
LDFLAGS = -static -T SnakeBW.lds --gc-sections
OBJ = SnakeBW.o game.o syscall.o
TARGET = SnakeBW.elf

all: $(TARGET)

%.o: %.c
	$(CC) $(CFLAGS) -c $<

syscall.o: ../betawise-sdk/syscall.c
	$(CC) $(CFLAGS) -c $< -o $@

$(TARGET): $(OBJ) SnakeBW.lds
	$(LD) $(OBJ) $(LDFLAGS) -o $@

clean:
	rm -f SnakeBW.o game.o syscall.o SnakeBW.elf
