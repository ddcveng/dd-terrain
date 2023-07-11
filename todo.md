# TODO
---------------- [REQUIRED] --------------------
[THESIS]
- check thesis for needed bibliography references

- add images
    - pixel art textures
    - showcase problems with arches version of texturing
    - creating the mesh - visualize our approach of making evlauation independent
    - showcase before after images somewhere
    - image from arches somewhere?

- redo measurements with full chunk height!

[CODE]
- make smooth mesh generation controllable at runtime via  imgui
    - polygonize and sdf sample will take parameters via input not via constants.
    - only update parameters when a button is pressed

- camera singularity bug
- make sure negative coords work
- support full chunk height -> this one kind of works already?
- black regions in smooth mesh bug

------------------- [NICE TO HAVE] ----------------------
- rigid block rendering
    - check how other trees and houses are rendered
    - maybe it will be better to only use sdfs to render the critical blocks and the rest will stay as plain old boxes

- document  concurrent loading in developer documentation

- shadow mapping
- screen space ambient occlusion

------------------ [NOT NEEDED] -------------------
- figure out a way to further reduce the amount of surface blocks.
    - this may not be that much of a problem, the app still runs at ~80 fps ...
    - check visibility globally
    - only render chunks that are in view frustum
    - partition chunks vertically also !! this is probably the best solution -> we render 10x10 chunk on surface (160x160 blocks) but render it 384 blocks deep.
        The vast majority of the blocks are never seen.
        -> maybe add an option to render only from some Y height up as a quick fix
- add more blocks
