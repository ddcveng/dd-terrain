## Running the app
Running the app requires a rust installer that supports the 2021 version of rust.
It is recommended to install [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) which is used for managing dependencies
and running rust projects.

After that you can just run `cargo run --release` in the root of the repository and cargo
will automatically resolve all dependencies and build and run the project. 
The initial run will take a little longer since all the dependencies have to be installed.

## Configuration
There is a number of variables that can be tweaked to modify how the render looks.
They can be found in the `config.rs` source file.

| Option            | Type    | Description                                                            |
|-------------------|---------|------------------------------------------------------------------------|
| WORLD_FOLDER      | string  | The path of the minecraft save file to be loaded                       |
| SPAWN_POINT       | vec3    | The position in the world where the camera is placed on startup        |
| WORLD_SIZE        | int     | A number N. Only a NxN region of chunks is loaded at a time            |
| CAMERA_MOVE_SPEED | float   | How fast the camera moves                                              |
| SENSITIVITY       | float   | How fast the camera turns                                              |
| ASSETS_PATH       | string  | The path to the folder containing textures and other resources         |
| DYNAMIC_WORLD     | boolean | If true, new chunks get loaded around the camera on demand as it moves |

## Controls
You control the in-app camera using the standard `WASD` for movement **forward**, **left**, **back**, and **right** respectively. 
You can move down and up using `J` and `K`. 
To rotate the camera in place, use the mouse when the cursor is captured.
Use `SPACE` to toggle mouse capture.

To toggle between the discrete and implicit view use `U`(discrete view) and `I`(implicit view)

You can also press `B` to toggle the wireframe.

Pressing `Q` will exit the application
