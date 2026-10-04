# Verification record

Review date: **2026-10-07**. Verified locally on Windows with Rust 1.88.0 and Rust 1.85.0. This record describes checks performed during the portfolio review; it is not a hosted CI result or an independent audit certificate.

## Build and quality checks

| Check | Result |
| --- | --- |
| Rustfmt | Passed on the final source |
| Clippy with `-D warnings` | Passed for all targets |
| Locked release build | Passed on Rust 1.88.0 and 1.85.0 |
| Original unit tests | 2 passed; retained in `src/math.rs` |
| Temporary review unit checks | 11 additional checks passed on both toolchains before removal |
| CLI integration checks | Passed against the release executable |
| Required images | Four P3 files validated at 800×600 with exactly 480,000 RGB triplets each |
| Gallery | Eight PNG images converted from renderer output |

The temporary review checks covered cube entry/exit normals on all axes, sphere tangency and interior rays, cylinder sides and caps, plane parallel rays, Snell refraction and total internal reflection, degenerate cameras, negative checker coordinates, numerical wave normals and bounded intersections, rendering determinism, and write/flush error propagation.

CLI checks verified that ten controls changed output: textures, reflections, refractions, particles, fluids, supersampling, brightness, camera position, camera target, and field of view. They also checked invalid arguments, time-dependent particle/wave snapshots, stdout/file equivalence, missing output directories, validation before file creation, and byte-identical images using 1 versus 64 workers with every effect enabled.

Temporary verification source, scripts, and scratch renders were removed after execution as requested. The retained test suite contains the original two vector-math tests; it does not retain the broader temporary regression coverage described here. GitHub Actions runs the retained tests alongside formatting, linting, and release builds.

## Requirement coverage

| Requirement | Evidence |
| --- | --- |
| Sphere, cube, plane, cylinder | Analytical implementations in `src/geometry.rs`; the `all` presets include all four |
| Object positioning | Explicit centers, bounds, and plane point in `src/scene.rs`; examples in `DOCUMENTATION.md` |
| Camera movement and angle | `--camera`, `--target`, `--fov`, and `all-alt`; alternate preset matched an equivalent camera override |
| Variable brightness and shadows | Preset point lights, CLI override, and visibility rays bounded by light distance |
| Reduced resolution | CLI dimensions; small 40×30 integration renders validated |
| Four 800×600 images | `scenes/sphere.ppm`, `cube-plane.ppm`, `all.ppm`, and `all-alt.ppm` regenerated |
| Usage and construction documentation | README and scene-authoring documentation with actual type/field names |
| Texture bonus | Optional 3D procedural checker on basic materials |
| Reflection/refraction bonus | Optional recursive reflection and dielectric transmission; visible glass preview |
| Particle bonus | Optional 32-sphere fountain, deterministic ballistic trajectories, time snapshots |
| Fluid bonus | Optional procedural wave height field with analytical normals; time snapshots and water preview |

Particles and water are procedural scene effects, not coupled physical simulations. The optics and performance limits are stated in the README. All four mandatory scene layouts are preserved; regeneration incorporates the corrected shader, tone mapping, and supersampling.

## Performance measurement

Hardware: AMD Ryzen 5 7600X, 6 cores / 12 logical CPUs. Toolchain: Rust 1.88.0 on Windows. Each measurement ran the release executable with `--scene all --textures --reflections --samples 2` at 800×600, varying `--threads` between 1 and 12.

The median of five runs was **0.563 s** for one worker and **0.126 s** for twelve, approximately **4.5×** faster for this workload. Timings include image rendering, PPM formatting, and stdout capture, and exclude compilation. They do not measure GPU rendering, path tracing, or an earlier revision's performance.

## Verification limits

Linux and macOS were not executed locally. The configured GitHub Actions jobs must run after pushing to verify those platforms. The local tests establish behavior for the supplied small scenes, not exhaustive numerical robustness for arbitrary geometry. In particular, grazing wave hits, nested dielectrics, caustics, particle collisions, and fluid dynamics are outside the verified scope.

The author-provided specification, audit checklist, and reference image remain local review inputs ignored by Git. Documentation gallery images are renders from this repository.
