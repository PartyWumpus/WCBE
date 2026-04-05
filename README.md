# A befunge editor

A fast 64 bit befunge93 IDE with breakpoint support. Inspired largely by [BefunExec](https://github.com/Mikescher/BefunExec). Can run [in the web](https://partywumpus.github.io/befunge-editor/) or locally.
I estimate a speed of about 100MHz max on my CPU (with the position history turned off and with PGO, it lowers to 50MHz for normal use). It likely runs slower in the web, but is still fast.

https://github.com/user-attachments/assets/1dea6b09-69a3-4802-8f5a-d125a5439d34

## Running

### Web

Go [to this site](https://partywumpus.github.io/befunge-editor/).

### Local

If you have nix, you can run `nix develop` to get all depencencies, otherwise you should take a look inside flake.nix and figure out how to get the deps you need.

Then run: `cargo run --release`

## Features

- All of befunge93
- Very minimal befunge98
- Breakpoints & Watchpoints
- Effectively infinite fungespace (up to the signed integer limit)
- Supports (most of) the [befunge-with-graphics](https://github.com/Jachdich/befunge-with-graphics) operations

## Features I would like to add in future:

- Manage shortcuts better instead of duplicating code all the time
- Breakpoints that pause on value change
- Some of the preprocessor things from befunexec (break & watch, but not replace)
- A better way to move the screen large distances. Possibly a "minimap" style thing?
- Undo in play mode
- Profile file threads
- Configurable "what to do on # at program edge"

## Thanks

Thanks [Mikescher](https://github.com/Mikescher) for the great BefunExec, and for the `Windmill` example program I stole for a preset program :)
