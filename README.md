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

# Problems/Issues

## Structure

I can't figure out how to handle and store chunks and blocks. What I want for sure is for blocks that are a part of systems to all run in parallel in the the ECS. I want to make the game extensible to new kinds of blocks and systems, and I figured that could take the form of there being new `Component`s introduced by other mods that can be attached to the proper block entites.

A `Chunk` has blocks, and a `Dimension` has chunks. I think it would be good for both of these to be components of entities too, since they can have rendering or time systems. 

The problem is that I need to be able to access a block given a dimension and coordinates. What I could do is have the dimension convert a given logical coordinates to a chunk and chunk offset. But then what is the chunk storing? The best I can think of are entity IDs. But the reason you want a block is you want one of its components. In order to get a block's component just by its entity ID, (as far as I know) you need to run a query for that component, and then check if the query contains the entity ID you have. That feels kind of inefficient to me, but I don't think there's a (good) way to keep a reference to a `Component`, especially since we need it to be mutable. 

The other problem with this is that you end up creating this `Dimension` that is going to get locked up when one thing needs to access it, if the thing is mutable. You would probably have some `DimensionStore` for all your dimensions, so that would get clogged too. You would want a (fast) way to access them concurrently. I think per chunk would be reasonable. So you would need to know the chunk you need and borrow that mutably? 

I'm kind of losing myself in this. There is a package that I need to look at for this. I'm realizing that:

1. I might be biting off more than I can chew
2. Bevy might not be the best choice for making this kind of game

Leaving 1. to the side, it comes back to my original goal of making an extensible voxel game that uses an ECS. The only major engines with ECS these days are Unity, which has kind of been a crap show, and Bevy. In a way Godot sort of has an ECS, but I don't think I want to get into using Godot the wrong way on my first time with it. I think for making things as extensible as possible, being able to just code the game in Rust is good too. So this brings me back to using Bevy.

If only there were some decent packages that would do some of this work for me, but alas, there aren't that many Bevy packages, especially ones that keep up with Bevy releases. 

I think that in the end, I am left to figure out how to make this work with Bevy, but I might be in over my head a little bit. I can take some time to explore packages that would help me, but I might need to find a way to just do this. It would be nice if there were at least some good examples for what I want to do, but it seems like every Bevy voxel example is for worlds with no block systems (so no blocks in the ECS), and every voxel engine plugin I look at for another system is not compatible with an ECS.

It makes me think I should just write my own voxel Bevy package before I do the rest of this project. It seems like the first thing that needs to be done. 
