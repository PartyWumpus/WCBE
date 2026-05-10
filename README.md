# WCBE (Wumpus' Concurrent Befunge Editor)

A fast 64 bit befunge93/98 IDE with breakpoint support. Inspired largely by [BefunExec](https://github.com/Mikescher/BefunExec). Can run [in the web](https://partywumpus.github.io/befunge-editor/) or locally.
I estimate a speed of about 100MHz max on my CPU (in b93 with the position history turned off and with PGO, it lowers to 50MHz for normal use). It likely runs slower in the web, but is still fast.
Befunge98 is not as fast.

https://github.com/user-attachments/assets/1dea6b09-69a3-4802-8f5a-d125a5439d34

## Running

### Web

Go [to this site](https://partywumpus.github.io/befunge-editor/).

### Local

If you have nix, you can run `nix develop` to get all dependencies, otherwise you should take a look inside flake.nix and figure out how to get the deps you need.

Then run: `cargo run --release`

For hot reloading: `dx serve --hotpatch --target x86_64-unknown-linux-gnu --bin WCBE`
For web build: `trunk build --release`
For standlone lib: `wasm-pack build --target web`

## Features

- All of befunge93
- Concurrent befunge98 (minus file reading/writing (`i`/`o`) and exec (`=`))
- Breakpoints & Watchpoints
- Effectively infinite fungespace (up to the signed integer limit)
- Supports (most of) the [befunge-with-graphics](https://github.com/Jachdich/befunge-with-graphics) operations in bf93

## Features I would like to add in future:

- Finish CLI support
- Manage shortcuts better instead of duplicating code all the time in app.rs
- Breakpoints that pause on value change
- Some of the preprocessor things from befunexec (break & watch, but not replace)
- A better way to move the screen large distances. Possibly a "minimap" style thing?
- Undo in play mode
- Profile file threads
- Configurable "what to do on # at program edge"
- Reduce crashes/infinite loops in b98
- Be fully conformant with mycology.b98 (almost done)
    - Shrink containing rect when spaces are placed at limits
    - Add latin-1 parsing mode
- `q` should _show_ its return value somewhere
- More b98 fingerprints

## Supported befunge98 fingerprints

### All platforms

- [NULL](https://catseye.tc/view/funge-98/library/NULL.markdown)
- [ROMA](https://catseye.tc/view/funge-98/library/ROMA.markdown)
- [BOOL](https://web.archive.org/web/20221029185454/http://www.rcfunge98.com/rcsfingers.html#bool)
- [MODU](https://catseye.tc/view/funge-98/library/MODU.markdown)
- [REFC](https://catseye.tc/view/funge-98/library/REFC.markdown)
- [FPDP](https://web.archive.org/web/20221029185454/http://www.rcfunge98.com/rcsfingers.html#fpsp)
- [FPSP](https://web.archive.org/web/20221029185454/http://www.rcfunge98.com/rcsfingers.html#fpdp)
- [FING](https://web.archive.org/web/20230617132045/https://rcfunge98.com/rcsfingers.html#fing)

### Not web 
- [HRTI](https://catseye.tc/view/funge-98/library/HRTI.markdown) (this is fixable)

## Thanks

Thanks [Mikescher](https://github.com/Mikescher) for the great BefunExec :)
