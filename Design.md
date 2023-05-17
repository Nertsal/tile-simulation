# Tile Simulation

The idea is to simulate real physics. The simplification that should make it possible is to split simulation into discrete tiles.

## Gravity and smooth movement

Often in discrete simulations the gravity is "hardcoded" in the sense that for every tile, its possible movement directions are limited.
For example, sand can only move down (straight or diagonally), and water can, in addition, move horizontally.

However, that does not allow for tiles to move up (i.e. due to being pushed).
The solution used here is to apply gravity for tiles every tick, changing their velocity.
Then, tiles move according to their velocities (and other properties, like friction).
This kind of gravity implies having non-discrete velocity, meaning that tiles can move by more than one tile every tick.

## Connected vessels or implementing pressure

In most tile simulations connected vessels simply do not work the same way they do in real life.
The reason is that water (or other liquids or gases) does not apply pressure to the surrounding tiles.
So the solution is to apply pressure, simple, right?
