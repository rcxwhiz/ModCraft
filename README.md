# ModCraft

This is a toy project of mine where I'm aiming to at least partially recreate a popular block-based game with performance and modding in mind. This game uses [bevy](https://bevyengine.org/) as an engine.

# Blocking issue

I have recently come against a blocking issue in my development. One of the key parts of this project for me is the ability to load the game and mods without compiling. I don't think you can pretend a game is useful or real if it requires every user to compile from source, and then additionally compile all mods from source. It also doesn't make for a very good modding API. 

The issue is that I want to be able to configure the bevy schedule at runtime, and allowe mods to wrap or replace vanilla systems. The way systems work right now is they are `IntoSystemConfigs`, which as far as I can tell are not allowed to be used with `dyn` or anything, meaning your systems have to be known at compile time. Even without mods, I was thinking this could be useful in my server code because there are sets of systems that are the same between the internal and dedicated servers, but they are added to different schedule labels. 

In my mind what would be optimal for modding is if mods could:
- Have access to simple APIs with things like `register_block(...)`
- Be allowed to provide plugins that can be registered with the `App`
- Have access to a structure that represents the vanilla systems and add/remove/replace/wrap them for ultimate control. 

It looks like this kind of thing is not so possible at the moment. There is the `bevy_dynamic_plugin` crate, which lets you have dynamic `Plugin`s, but it's a little hacky and delicate. There is a feature on track for release in `bevy 0.13` with a revamped query system that would maybe take away the pain of the `IntoSystemConfigs` stuff. It seems like in general there aren't a lot of dynamic features in Rust or Bevy, so this project in general might be a bit of a dead end. 

# Short Term Goals

- [X] Workspace for alpha development
- [X] Client/server compilation options
- [X] Figure out split between server/client/lib architecture 
- [X] Get bevy setup in the workspace
- [X] Pick client/server communication library (probably `bevy_quinnet` or `bevy_renet`) (chose `bevy_quinnet` for now)
- [X] Get `bevy_quinnet integrated into project`
- [X] Get client-server architecture resolved
- [ ] Figure out how to make system sets easier that are reusable
- [ ] Figure out how to make an equivalent non-network message queue between client and internal server in singleplayer
- [ ] Use a dynamic library loading crate to load a mod (`libloading`?)
- [ ] Get `modcraft_lib` to be able to define a mod
- [ ] An example mod that loads and runs
- [ ] Figure out the real appropriate license

# Long Term Goals

- [ ] Config options built into `modcraft_lib` (should that be split into its own crate?)
