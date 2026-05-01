# SmartApplets

Repo-owned workflow for reliable AlphaSmart NEO SmartApplets using the validated
Betawise-derived C path.

## One-command build

From the repo root:

```sh
./scripts/build-smartapplet.sh basic_writer_bw
```

That command does all of the working steps:

1. builds the applet ELF with `m68k-elf-gcc` / `m68k-elf-ld`
2. packs the ELF with local `alpha-neo-pack`
3. writes the final `.os3kapp` into `exports/applets/`
4. runs the headless emulator validator

To launch the GUI emulator after a successful build:

```sh
./scripts/build-smartapplet.sh basic_writer_bw --gui
```

To skip validation:

```sh
./scripts/build-smartapplet.sh basic_writer_bw --no-validate
```

The same workflow applies to every applet with an `applet.env`, for example:

```sh
./scripts/build-smartapplet.sh write_or_die_bw
```

## Tool requirements

On macOS with Homebrew:

```sh
brew install m68k-elf-binutils m68k-elf-gcc
```

## Writing applets this way

Use these rules. They come directly from the working path:

- keep Betawise syscall stubs and headers
- do not use Betawise `os3k.c`
- use a custom no-global applet runtime
- store all mutable applet state at `A5 + 0x300`
- pack the final ELF with local `alpha-neo-pack`
- for applet-owned persistence, use the proven runtime file handle and trap path,
  not guessed metadata ids

In practice that means:

- include `smartapplets/betawise-sdk/os3k.h`
- include `smartapplets/betawise-sdk/applet.h` for the entry shim, state macro,
  and shared applet status constants
- include `smartapplets/betawise-sdk/file_store.h` for the validated one-file
  snapshot persistence path
- link `smartapplets/betawise-sdk/syscall.c`
- link `smartapplets/betawise-sdk/file_store.c` if the applet persists state
- provide your own entry shim in `.text.alpha_usb_entry`
- dispatch `MSG_SETFOCUS`, `MSG_CHAR`, and `MSG_KEY` yourself
- handle Enter as `MSG_CHAR` byte `0x0d`/`0x0a` where the screen flow expects
  a confirmation key; under full firmware dispatch it is not guaranteed to
  arrive as `MSG_KEY KEY_ENTER`
- do not rely on writable global C variables
- keep the linker script simple and discard `.bss`, `.data`, `.got`, `.rela`, `.rel`

## Shared SDK pieces

The repo-owned Betawise-side SDK now exposes these validated low-level helpers:

- `applet.h`
  - `APPLET_ENTRY(handler)`
  - `APPLET_STATE(Type)`
  - `APPLET_EXIT_STATUS`
  - `APPLET_UNHANDLED_STATUS`
- `file_store.h` / `file_store.c`
  - `applet_load_snapshot(...)`
  - `applet_save_snapshot(...)`

This means applet code no longer needs to carry:

- the raw `alpha_usb_entry` assembly shim
- direct `A5 + 0x300` pointer boilerplate
- direct `A2DC -> A2EC/A2FC -> A190 -> FileReadBuffer/FileWriteBuffer -> FileClose`
  choreography for one-file snapshot persistence

For the currently validated one-file persistence path used by `forth_mini_bw`:

- runtime file handle is `1`
- workspace/file sequence is `A2DC -> A2EC/A2FC -> A190 -> FileReadBuffer/FileWriteBuffer -> FileClose`
- persisting a binary app state or machine snapshot is safer than replaying source on target when the code uses struct returns under `-fshort-enums`

## Compiler profiles

There is a real codegen boundary here.

For small, straightforward applets such as `basic_writer_bw`, the default compact
profile is fine:

```make
CFLAGS += -Os
```

For more complex applets with denser control flow or REPL-style logic such as
`forth_mini_bw`, use the conservative profile instead:

```make
CFLAGS += -O1 -fno-inline -fno-optimize-sibling-calls
```

Why this matters:

- `forth_mini_bw` built successfully with more aggressive optimization
- but the resulting applet crashed at runtime under real firmware dispatch
- backing optimization down and disabling inlining/tail-call shaping made the
  applet stable in the emulator validation path

So the rule is:

- start with the compact profile for simple applets
- switch to the conservative profile as soon as a complex applet shows unstable
  runtime behavior that is not explained by the applet logic itself

## Layout

Each applet lives in `smartapplets/<name>/` and should contain:

- applet source, usually `<Name>.c`
- linker script
- `Makefile`
- `applet.env`

`applet.env` tells the wrapper script how to:

- locate the built ELF
- choose the `alpha-neo-pack` manifest
- choose the output `.os3kapp`
- run an applet-specific validator

## Minimal state pattern

The validated pattern is:

```c
typedef struct {
    /* applet state */
} AppState_t;

static inline AppState_t* State(void) {
    register char* a5 __asm__("a5");
    return (AppState_t*)(a5 + 0x300);
}
```

Everything mutable should hang off that state block.

## Current references

- `smartapplets/basic_writer_bw`: simple editor applet, good default template
- `smartapplets/forth_mini_bw`: more complex REPL applet, reference for the
  conservative compiler profile
- `smartapplets/write_or_die_bw`: challenge editor applet, reference for setup
  menus, live timers, pressure states, one-file autosave, and full-system
  headless validation of completion plus persistence
