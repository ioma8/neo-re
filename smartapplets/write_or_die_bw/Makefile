CC = m68k-elf-gcc
LD = m68k-elf-ld

CFLAGS = -m68000 -mpcrel -fomit-frame-pointer -ffixed-a5 -ffixed-d7 -fshort-enums
CFLAGS += -O1 -fno-inline -fno-optimize-sibling-calls -I"../betawise-sdk" -ffunction-sections -fdata-sections -ffreestanding
LDFLAGS = -static -T WriteOrDieBW.lds --gc-sections
OBJ = WriteOrDieBW.o editor.o challenge.o ui.o storage.o alpha_export.o syscall.o bw_store.o bw_screen.o bw_challenge.o bw_alphaword.o
TARGET = WriteOrDieBW.elf

all: $(TARGET)

%.o: %.c
	$(CC) $(CFLAGS) -c $<

syscall.o: ../betawise-sdk/syscall.c
	$(CC) $(CFLAGS) -c $< -o $@

bw_store.o: ../betawise-sdk/bw_store.c
	$(CC) $(CFLAGS) -c $< -o $@

bw_screen.o: ../betawise-sdk/bw_screen.c
	$(CC) $(CFLAGS) -c $< -o $@

bw_challenge.o: ../betawise-sdk/bw_challenge.c
	$(CC) $(CFLAGS) -c $< -o $@

bw_alphaword.o: ../betawise-sdk/bw_alphaword.c
	$(CC) $(CFLAGS) -c $< -o $@

$(TARGET): $(OBJ) WriteOrDieBW.lds
	$(LD) $(OBJ) $(LDFLAGS) -o $@

clean:
	rm -f WriteOrDieBW.o editor.o challenge.o ui.o storage.o alpha_export.o syscall.o bw_store.o bw_screen.o bw_challenge.o bw_alphaword.o WriteOrDieBW.elf
