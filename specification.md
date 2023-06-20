# DDTerrain

The goal of this project is implementing an application
that can render discrete volumetric data as as a smooth continuous surface.
Specifically the focus will be on rendering volumetric terrain that can be used
in a videogame or a render of some sort.

We will use the blocky worlds of minecraft the video game as input data, since they are easy to get, have infinite variations and produce nice output.

## Requirements
- real-time exploration of the rendered scene from the point of view of a flying
perspective camera
- real-time loading of world data with smooth movement around the world
    - Minecraft worlds are infinite so we only render a small portion at a time 
    and load/unload chunks on demand
- rendering of the **discrete** input data
- rendering of the **smooth** terrain
    - configurable smoothness
- textured smooth terrain
    - controllable texture blending
- configurable "rigid" blocks
    - blocks that will not be translated into smooth terrain but will stay as they are (tree trunks, buildings, ...)
    - rendering of rigid + smooth terrain with nice transitions between the two

## Comparison to existing work
Many techniques for polygonizing volumetric data exist. The main inspirations for this project are the 2009 paper Arches: A framework for modeling complex terrains by Peytavie et al. and a mod for minecraft called "NoCubes".

We will use the algorithm for extracting the smooth surface from *Arches* as well as the technique for texturing this surface.

We share a lot of ideas with the NoCubes mod, but our implementation is more general and works with any volumetric data, not just minecraft worlds.

## Solution
We will use the algorithm described in *Arches* to extract a potential function `f(p) -> R+` from the input data. This function maps each point in 3D space to a real value from the range `<0, 1>` - the "density" of the volume at that point. The density is then transformed to the range `<-1, 1>` where higher value means lower density. In this form, the function can be used as a **signed distance function**.

A *signed distance function* (or **sdf**) is a function `f(p) -> R` that for each point in 3D space returns the **signed distance** to a object, with *negative* values for points inside the object. The set of points `S = {x | f(x) == a}` is then called the *a-isosurface* of f.

We will want to visualize the 0-isosurface of our density function. 
Our density function is not an entirely valid sdf, since it will return 1 for all points with distance >= 1, but we only care about the neighborhood of 0 for which we have valid values.

There are many ways to visualize an sdf, we chose a method called **MarchingCubes** for its low complexity and relatively high quality results. It works by building a polygon mesh of the isosurface that is then rendered using the standard rendering pipeline.

### The floating tree problem
Extracting the sdf from the discrete data shrinks the world a little (depends on how much we smooth it out). This means that since rigid blocks will stay where they are, the ground will (sometimes) disappear right from under them and they will be left there floating.

Imagine a tree standing on the edge of a cliff. The tree is set to be rigid, meaning it will not be smoothed out along with the terrain and stay as is. This is where the problem arises; since the cliff will become rounded and shrunk, the tree is now floating in midair. Thus the floating tree problem.

We can solve it by using sdf's. To get the union of 2 sdf's, we can just take the **minimum** of the distances. We will join the terrain sdf with the sdf of a box - our rigid block.
This will get us what we want, but point where the objects meet will have a hard jump from one object to the other. In our case, we want to merge smooth terrain with a not-so smooth cube which will look out of place just clipping throught the ground. 

We would like to have a function that behaves like standard *min* when the values are far away but does some kind of smooth easing from one value to the other when they start to get close. One such function is the smoothmin function described [here](https://iquilezles.org/articles/smin/) by Inigo Quilez.

With this we can have both smooth terrain and rigid blocks with smooth transitions between the two.

## Implementation
The result will be a standalone graphical application that can extract sdf's from discrete volumetric data and render them on screen.

The renderer will be written from scratch using the `OpenGL` API for rendering the graphics.

Our language of choice is the `Rust` programming language which we will be using together with the `Cargo` build system and package manager to manage dependencies and building of the project.

We will be using a higher abstraction level library interfacing with OpenGL called `glium`. It provides type safe bindings to the low level C API and supports both Windows and Linux, so our application supports both platforms for free.

### Dependencies
- [glium](https://crates.io/crates/glium)
    - a higher level abstraction over the OpenGL API
- [cgmath](https://crates.io/crates/cgmath)
    - maths library for 3D graphics
- [imgui](https://crates.io/crates/imgui)
    - for creating simple UI elements
- [fastanvil](https://crates.io/crates/fastanvil)
    - for working with the minecraft save file format - [anvil](https://minecraft.fandom.com/wiki/Anvil_file_format)