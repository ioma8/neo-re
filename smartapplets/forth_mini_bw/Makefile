CC = m68k-elf-gcc
LD = m68k-elf-ld

CFLAGS = -m68000 -mpcrel -fomit-frame-pointer -ffixed-a5 -ffixed-d7 -fshort-enums
CFLAGS += -O1 -fno-inline -fno-optimize-sibling-calls -I"../betawise-sdk" -ffunction-sections -fdata-sections -ffreestanding
LDFLAGS = -static -T ForthMiniBW.lds --gc-sections
OBJ = ForthMiniBW.o syscall.o
TARGET = ForthMiniBW.elf

all: $(TARGET)

%.o: %.c
	$(CC) $(CFLAGS) -c $<

syscall.o: ../betawise-sdk/syscall.c
	$(CC) $(CFLAGS) -c $< -o $@

$(TARGET): $(OBJ) ForthMiniBW.lds
	$(LD) $(OBJ) $(LDFLAGS) -o $@

clean:
	rm -f ForthMiniBW.o syscall.o ForthMiniBW.elf
