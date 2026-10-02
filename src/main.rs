mod camera;
mod cli;
mod effects;
mod geometry;
mod math;
mod renderer;
mod scene;

use cli::{parse_args, print_help};
use renderer::{RenderOptions, render};
use scene::{add_effects, scene_by_name};
use std::fs::File;
use std::io::{self, BufWriter};
use std::process;

fn run() -> Result<(), String> {
    let args = parse_args()?;
    if args.help {
        print_help();
        return Ok(());
    }

    let mut scene = scene_by_name(
        &args.scene,
        args.brightness,
        args.camera,
        args.target,
        args.fov,
    )?;
    add_effects(
        &mut scene,
        args.refractions,
        args.particles,
        args.fluids,
        args.time,
    );
    let options = RenderOptions {
        width: args.width,
        height: args.height,
        reflections: args.reflections,
        refractions: args.refractions,
        textures: args.textures,
        max_depth: args.max_depth,
        samples: args.samples,
        threads: args.threads,
    };

    match args.output {
        Some(path) => {
            let file =
                File::create(&path).map_err(|err| format!("failed to create {path}: {err}"))?;
            render(&scene, &options, BufWriter::new(file))
                .map_err(|err| format!("failed to render {path}: {err}"))?;
        }
        None => {
            let stdout = io::stdout();
            render(&scene, &options, BufWriter::new(stdout.lock()))
                .map_err(|err| format!("failed to write ppm: {err}"))?;
        }
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("rt: {err}");
        process::exit(1);
    }
}
