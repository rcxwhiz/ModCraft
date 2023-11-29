# ModCraft

This is a toy project of mine where I'm aiming to at least partially recreate a popular block-based game with performance and modding in mind. This game uses [bevy](https://bevyengine.org/) as an engine.

`modcraft_app` is the crate for the actual game. It has an optional `client` feature flag that will compile a client rather than a standalone app. It depends on `modcraft_lib`. The idea is that `modcraft_lib` will actually contain the majority of the code, and it will be the library that "mods" depend on.

# Short Term Goals

- [X] Workspace for alpha development
- [X] Client/server compilation options
- [ ] Figure out split between server/client/lib architecture 
- [ ] Get bevy setup in the workspace
- [ ] Pick client/server communication library (probably `bevy_quinnet` or `bevy_renet`)
- [ ] Use a dynamic library loading crate to load a mod (`libloading`?)
- [ ] Get `modcraft_lib` to be able to define a mod
- [ ] An example mod that loads and runs
