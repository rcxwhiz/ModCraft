# ModCraft

This is a toy project of mine where I'm aiming to at least partially recreate a popular block-based game with performance
and modding in mind. This game uses [bevy](https://bevyengine.org/) as an engine.

# Build Instructions

As of now this project is a standard, stand alone bevy game. The `Cargo.toml` file is configured according to [the basic
tutorial provided by bevy](https://bevyengine.org/learn/book/getting-started/setup/). If you would like to have faster compiles, use the `--features bevy/dynamic_linking` flag, ex.
`cargo run --features bevy/dynamic_linking`.

This project was created with rust `1.73.0`, but you should assume it uses all the features in the latest stable build
of rust.

# Goals / Roadmap

One of my main goals of this game is to have first class modding. I plan to do this by having a separate API create, and
then dynamically load mods with the game executable. I have not made any of this functionality yet due to having no
experience whatsoever writing rendering code. I have a real computer science degree, but part of the fun of this project
for me is that we never directly covered any of these topics in my classes. 
