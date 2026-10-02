use crate::math::Vec3;
use std::{env, str::FromStr};

pub(crate) struct Args {
    pub(crate) scene: String,
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) output: Option<String>,
    pub(crate) brightness: Option<f64>,
    pub(crate) camera: Option<Vec3>,
    pub(crate) target: Option<Vec3>,
    pub(crate) fov: Option<f64>,
    pub(crate) reflections: bool,
    pub(crate) refractions: bool,
    pub(crate) textures: bool,
    pub(crate) particles: bool,
    pub(crate) fluids: bool,
    pub(crate) time: f64,
    pub(crate) samples: usize,
    pub(crate) threads: usize,
    pub(crate) max_depth: usize,
    pub(crate) help: bool,
}

pub(crate) fn parse_args() -> Result<Args, String> {
    parse(env::args().skip(1))
}

pub(crate) fn parse(mut iter: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut args = Args {
        scene: "sphere".into(),
        width: 800,
        height: 600,
        output: None,
        brightness: None,
        camera: None,
        target: None,
        fov: None,
        reflections: false,
        refractions: false,
        textures: false,
        particles: false,
        fluids: false,
        time: 0.0,
        samples: 1,
        threads: std::thread::available_parallelism().map_or(1, |n| n.get().min(64)),
        max_depth: 5,
        help: false,
    };
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--scene" => args.scene = next_value(&mut iter, "--scene")?,
            "--width" => args.width = parse_next(&mut iter, "--width")?,
            "--height" => args.height = parse_next(&mut iter, "--height")?,
            "--output" | "-o" => args.output = Some(next_value(&mut iter, "--output")?),
            "--brightness" => args.brightness = Some(parse_next(&mut iter, "--brightness")?),
            "--camera" => args.camera = Some(parse_vec3(&next_value(&mut iter, "--camera")?)?),
            "--target" => args.target = Some(parse_vec3(&next_value(&mut iter, "--target")?)?),
            "--fov" => args.fov = Some(parse_next(&mut iter, "--fov")?),
            "--samples" => args.samples = parse_next(&mut iter, "--samples")?,
            "--threads" => args.threads = parse_next(&mut iter, "--threads")?,
            "--max-depth" => args.max_depth = parse_next(&mut iter, "--max-depth")?,
            "--time" => args.time = parse_next(&mut iter, "--time")?,
            "--reflections" | "-r" => args.reflections = true,
            "--refractions" => args.refractions = true,
            "--textures" | "-t" => args.textures = true,
            "--particles" => args.particles = true,
            "--fluids" => args.fluids = true,
            "--help" | "-h" => {
                args.help = true;
                return Ok(args);
            }
            unknown => {
                return Err(format!(
                    "unknown argument: {unknown}. Use --help for options."
                ));
            }
        }
    }
    if args.width == 0
        || args.height == 0
        || args.width > 16384
        || args.height > 16384
        || args
            .width
            .checked_mul(args.height)
            .is_none_or(|n| n > 16_777_216)
    {
        return Err("dimensions must be 1..16384 with at most 16,777,216 pixels".into());
    }
    if !(1..=8).contains(&args.samples) {
        return Err("samples must be 1..8 per axis".into());
    }
    if !(1..=64).contains(&args.threads) {
        return Err("threads must be 1..64".into());
    }
    if !(1..=10).contains(&args.max_depth) {
        return Err("max-depth must be 1..10".into());
    }
    if args
        .brightness
        .is_some_and(|n| !n.is_finite() || !(0.0..=1000.0).contains(&n))
    {
        return Err("brightness must be finite and between 0 and 1000".into());
    }
    if args
        .fov
        .is_some_and(|n| !n.is_finite() || !(1.0..179.0).contains(&n))
    {
        return Err("fov must be at least 1 and less than 179 degrees".into());
    }
    if !args.time.is_finite() || !(0.0..=1_000_000.0).contains(&args.time) {
        return Err("time must be finite and between 0 and 1,000,000 seconds".into());
    }
    Ok(args)
}

fn next_value(iter: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    iter.next()
        .filter(|s| !s.is_empty() && !s.starts_with("--"))
        .ok_or_else(|| format!("missing value after {flag}"))
}

fn parse_next<T: FromStr>(
    iter: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<T, String> {
    let value = next_value(iter, flag)?;
    value
        .parse()
        .map_err(|_| format!("invalid value '{value}' for {flag}"))
}

fn parse_vec3(value: &str) -> Result<Vec3, String> {
    let parts: Vec<_> = value.split(',').map(str::trim).collect();
    if parts.len() != 3 {
        return Err("coordinates must use x,y,z format, for example 3,2,5".into());
    }
    let coord = |part: &str| -> Result<f64, String> {
        let n: f64 = part
            .parse()
            .map_err(|_| format!("invalid coordinate '{part}'"))?;
        if !n.is_finite() || n.abs() > 1_000_000.0 {
            return Err("coordinates must be finite and within +/-1,000,000".into());
        }
        Ok(n)
    };
    Ok(Vec3::new(
        coord(parts[0])?,
        coord(parts[1])?,
        coord(parts[2])?,
    ))
}

pub(crate) fn print_help() {
    println!(
        "rt\n\
Usage: rt [options]\n\n\
  --scene <name>          sphere | cube-plane | all | all-alt (default: sphere)\n\
  --width <pixels>        Width (default: 800)\n\
  --height <pixels>       Height (default: 600)\n\
  --output, -o <file>     Write ASCII P3 PPM to file (default: stdout)\n\
  --brightness <number>   Light brightness, 0..1000\n\
  --camera <x,y,z>        Camera position\n\
  --target <x,y,z>        Camera look-at target\n\
  --fov <degrees>        Vertical field of view, 1 <= fov < 179\n\
  --reflections, -r      Reflective materials\n\
  --refractions          Glass sphere and water transmission, including Fresnel reflection\n\
  --textures, -t         Procedural checker textures\n\
  --particles            Add 32 ballistic particles\n\
  --fluids               Add a procedural water surface\n\
  --time <seconds>       Particle and wave snapshot time (default: 0)\n\
  --samples <count>      Grid samples per axis, 1..8 (default: 1)\n\
  --threads <count>      Workers, 1..64 (default: logical CPU count, capped at 64)\n\
  --max-depth <count>    Reflection/refraction bounce limit, 1..10 (default: 5)\n\
  --help, -h             Show help\n\n\
Example: cargo run --release -- --scene all --textures --reflections -o output.ppm"
    );
}
