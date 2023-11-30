# ModCraft

This is a toy project of mine where I'm aiming to at least partially recreate a popular block-based game with performance and modding in mind. This game uses [bevy](https://bevyengine.org/) as an engine.

## Crates

`modcraft_app`: Contains a compilation feature flag `client` which will compile the crate into a client vs. a server for the game. On the client side, contains code for rendering and input handling and sending data to the server. When playing single player, the client can start a process with a server. The server code handles game logic and interaction with clients via networking.

`modcraft_lib`: Contains everything(?) that a mod would need to depend on to compile into a library that can be dynamically loaded at runtime by the client or server. The server crate depends on this library and uses it to load anonymous mods at runtime.

`example_mod`: An example mod crate that depends on `modcraft_lib`.

# Short Term Goals

- [X] Workspace for alpha development
- [X] Client/server compilation options
- [X] Figure out split between server/client/lib architecture 
- [X] Get bevy setup in the workspace
- [ ] Pick client/server communication library (probably `bevy_quinnet` or `bevy_renet`)
- [ ] Use a dynamic library loading crate to load a mod (`libloading`?)
- [ ] Get `modcraft_lib` to be able to define a mod
- [ ] An example mod that loads and runs

# Long Term Goals

- [ ] Config options built into `modcraft_lib` (should that be split into its own crate?)
