mod camera;
mod cli;
mod geometry;
mod math;
mod renderer;
mod scene;
use cli::{parse_args, print_help};
use renderer::{RenderOptions, render};
use scene::scene_by_name;
use std::fs::File;
use std::io::{self, BufWriter};
use std::process;

fn run() -> Result<(), String> {
    let args = parse_args()?;
    if args.help {
        print_help();
        return Ok(());
    }

    let scene = scene_by_name(&args.scene, args.brightness, args.camera).ok_or_else(|| {
        format!(
            "unknown scene '{}'. Use --help for valid scenes.",
            args.scene
        )
    })?;
    let options = RenderOptions {
        width: args.width,
        height: args.height,
        reflections: args.reflections,
        max_depth: 3,
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
