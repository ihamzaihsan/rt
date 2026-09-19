# rt documentation

`rt` is a small ray tracer that renders ASCII PPM images. It supports spheres,
cubes, flat planes, finite cylinders, movable cameras, light brightness,
shadows, and optional reflections.

## Running

Render the default 800x600 sphere scene to stdout:

```sh
cargo run --release > output.ppm
```

Render a specific scene to a file:

```sh
cargo run --release -- --scene all --reflections --output scenes/all.ppm
```

Useful options:

```text
--scene <name>          sphere | cube-plane | all | all-alt
--width <pixels>        output width, default 800
--height <pixels>       output height, default 600
--output, -o <file>     write to a file instead of stdout
--brightness <number>   override scene light brightness
--camera <x,y,z>        move the camera while keeping the scene target
--reflections, -r       enable reflective materials
```

Use a smaller image while experimenting:

```sh
cargo run --release -- --scene all --width 320 --height 240 --output test.ppm
```

## Features

The renderer casts one ray through each pixel from a camera into a 3D scene. It
finds the closest object intersection, shades that point with ambient and
diffuse light, checks whether another object blocks the light to create shadows,
and optionally follows reflection rays for shiny surfaces.

Objects are defined in `src/main.rs` with the `Object` enum. Every object owns a
`Material`, which contains an RGB color in the `0.0..1.0` range and a
reflectivity value.

## Creating Objects

Create a sphere:

```rust
Object::Sphere {
    center: Vec3::new(0.0, 1.0, 0.0),
    radius: 1.0,
    material: material(Vec3::new(0.95, 0.18, 0.14), 0.35),
}
```

Create a cube using minimum and maximum corners:

```rust
Object::Cube {
    min: Vec3::new(-0.8, -0.55, -0.8),
    max: Vec3::new(0.8, 1.05, 0.8),
    material: material(Vec3::new(0.15, 0.43, 0.82), 0.18),
}
```

Create a flat plane from one point on the plane and a normal:

```rust
Object::Plane {
    point: Vec3::new(0.0, 0.0, 0.0),
    normal: Vec3::new(0.0, 1.0, 0.0),
    material: material(Vec3::new(0.72, 0.74, 0.70), 0.12),
}
```

Create a vertical finite cylinder:

```rust
Object::Cylinder {
    center: Vec3::new(0.0, 0.05, -1.55),
    radius: 0.45,
    height: 1.6,
    material: material(Vec3::new(0.91, 0.68, 0.20), 0.24),
}
```

## Moving Objects

Change an object's position by changing its coordinates. For example, this
moves a sphere center to `(1, 1, 1)`:

```rust
Object::Sphere {
    center: Vec3::new(1.0, 1.0, 1.0),
    radius: 1.0,
    material: material(Vec3::new(0.95, 0.18, 0.14), 0.35),
}
```

For a cube, move both `min` and `max` by the same amount. For a plane, change
`point`. For a cylinder, change `center`.

## Changing Brightness

Each scene has one or more `Light` values:

```rust
Light {
    position: Vec3::new(-3.0, 5.5, 3.0),
    color: Vec3::new(1.0, 0.96, 0.88),
    brightness: 5.5,
}
```

Increase `brightness` for a stronger light or decrease it for a darker render.
You can also override brightness from the command line:

```sh
cargo run --release -- --scene cube-plane --brightness 2.4 --output dim.ppm
```

## Changing Camera Position And Angle

Cameras are created with `Camera::look_at`:

```rust
Camera::look_at(
    Vec3::new(3.7, 2.4, 5.0),     // camera position
    Vec3::new(0.0, 0.0, -0.35),   // target point
    Vec3::new(0.0, 1.0, 0.0),     // up direction
    50.0,                         // field of view in degrees
)
```

Move the camera by changing the first vector. Change the viewing angle by
changing the target point or the field of view. From the command line, move only
the camera position like this:

```sh
cargo run --release -- --scene all --camera -4.4,2.3,3.2 --output alt.ppm
```

## Required Evaluation Images

The project includes four required 800x600 PPM outputs in `scenes/`:

- `sphere.ppm`: a scene with a sphere
- `cube-plane.ppm`: a flat plane and a cube with lower brightness
- `all.ppm`: one cube, one sphere, one cylinder, and one flat plane
- `all-alt.ppm`: the same objects from another camera position
