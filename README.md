# Factorio blueprint processor

Various algorithms for generating weird and wonderful Factorio blueprints.

This code is somewhat disorganized, and I don't expect that I'll do the work to organize it – it should be viewed as an abandoned project. This readme is a short attempt to make it easier to read for newcomers.

## The files

`src/belt_routing.rs` is where the main fun is. It's an algorithm for routing multiple conveyors to-and-from assemblers or other conveyors.

`src/bin/sandbox.rs` contains some commented-out code that quantizes an image file and renders it as a grid of Factorio entities.

`src/blueprint.rs` is mostly the work of [notjack](http://github.com/jackfirth/rust-learning). It's for loading and saving the zlib-compressed JSON that is Factorio blueprint strings, to a straightforward struct representation. 

`src/simplified.rs` gives a simplified representation of some common Factorio entity types, used for my algorithms.

At the time of this writing, running `cargo run --bin sandbox` will just call the algorithms in `src/belt_routing.rs`, to generate and print out an advanced circuit assembly layout that's compatible with [notjack's gigabase framework](https://factorioprints.com/view/-LY5Lm5wbvM1gwtE1cSc).

`src/optimizer.rs` was a mostly failed attempt to route conveyors using hillclimbing rather than a real pathfinding algorithm.

## The main algorithm

We start by using an A* pathfinding algorithm to find a path from the source conveyor to one or more destinations.

If there's more than one destination, we stop the first time we reach a destination, and commit to the route found so far. Then we find every position where you could branch a splitter off of the route we've committed to so far, and run A* again, using those splitter locations as the new sources. Then repeat until all destinations of been reached.

The system can also be set to route conveyors "backwards", meaning many assemblers/conveyors can merge their outputs to one conveyor, instead of one conveyor splitting to many assemblers/conveyors.

To route multiple conveyors, we first run the algorithm with overlaps permitted – overlapping another conveyor only increases the *cost* (a.k.a. the A* edge weight) of the overlapping conveyors. Then we run multiple iterations of the algorithm, repeatedly increasing the overlap-cost until all but one of the conveyors choose a different route. There's also some special cases for when a conveyor overlaps a different part of its *own* route, to make sure we don't get stuck rushing to the first destination by a short route that has no positions to branch off from.
