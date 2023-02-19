# Nastroj na generovanie a editaciu volumetrickeho terenu
## Bakalarska praca

### Vyvojove nastroje
- C/C++ 
    - obyc C ma asi nejake performance vyhody + DOP ale C++20 bude ovela prijemnejsie
- Rust
    - Luminance / GLium
    - nemusim riesit cancer c veci, ale zato musim sa naucit rust
- GLSL latest
    - ked nebudem potrebovat latest features mozem znizit nech to bezi aj na starsich
    masinach ale nechcem sa tym obmadzovat
- Neovim lepsi setup na rust
- obsidian
    - notes
    - canban
    - whiteboard
- hyprland?
- RenderDoc - shader debugger ?
    - https://github.com/baldurk/renderdoc



### Hlavne casti projektu
- renderovanie sceny na GPU
    - pohybliva kamera
    - mesh/instancing?
    - rendering implicitne plochy
        - marching cubes
    - debug options
        - wireframe
    - optimalizovane
- reprezentacia dat 
    - kvazi voxely (neuniformna vyska bloku)
    - implicitne plochy -> toto mozno optional
- generovanie terenu
    - technika co pouzili oni
    - wave function collapse?
- editacia terenu
    - voxely
        - pridavanie/odoberanie voxelov (minecraft)
        - natahovanie do vysky
    - implicit 
        - pridavat/uberat material v smere normaly v danom bode
    - paintbrush na textury? - velka tazoba 90% nie
- textury
    - voxel - trivialne
    - implicit - tazoba(optional?)
- model sa da vyexportovat v nejakom rozumnom formate pre pouzitie v inom projekte

### Rocnikovy projekt
- procedural generation
- voxel terrain
- rendering
- editor

### Bakalarka
- vyhladenie do implicitnych ploch
- editor implicitnych ploch
- export do nejakeho 3d model formatu

Ref:
[Arches](https://www.researchgate.net/publication/227604236_Arches_a_Framework_for_Modeling_Complex_Terrains)
[GPU Gems 3 - Marching cubes](https://developer.nvidia.com/gpugems/gpugems3/part-i-geometry/chapter-1-generating-complex-procedural-terrains-using-gpu)
[Marching cubes - lookup table](http://paulbourke.net/geometry/polygonise/)

