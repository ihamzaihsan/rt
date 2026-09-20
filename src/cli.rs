use crate::math::Vec3;
use std::env;
#[derive(Default)]
pub(crate) struct Args {
    pub(crate) scene: String,
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) output: Option<String>,
    pub(crate) brightness: Option<f64>,
    pub(crate) camera: Option<Vec3>,
    pub(crate) reflections: bool,
    pub(crate) help: bool,
}

pub(crate) fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        scene: "sphere".to_string(),
        width: 800,
        height: 600,
        reflections: false,
        ..Args::default()
    };
    let mut iter = env::args().skip(1);

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--scene" => args.scene = next_value(&mut iter, "--scene")?,
            "--width" => args.width = parse_next(&mut iter, "--width")?,
            "--height" => args.height = parse_next(&mut iter, "--height")?,
            "--output" | "-o" => args.output = Some(next_value(&mut iter, "--output")?),
            "--brightness" => args.brightness = Some(parse_next(&mut iter, "--brightness")?),
            "--camera" => args.camera = Some(parse_vec3(&next_value(&mut iter, "--camera")?)?),
            "--reflections" | "-r" => args.reflections = true,
            "--help" | "-h" => args.help = true,
            unknown => return Err(format!("unknown argument: {unknown}")),
        }
    }

    if args.width == 0 || args.height == 0 {
        return Err("width and height must be greater than 0".to_string());
    }

    Ok(args)
}

pub(crate) fn next_value(
    iter: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<String, String> {
    iter.next()
        .ok_or_else(|| format!("missing value after {flag}"))
}

pub(crate) fn parse_next<T: std::str::FromStr>(
    iter: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<T, String> {
    next_value(iter, flag)?
        .parse()
        .map_err(|_| format!("invalid value for {flag}"))
}

pub(crate) fn parse_vec3(value: &str) -> Result<Vec3, String> {
    let parts: Vec<_> = value.split(',').collect();
    if parts.len() != 3 {
        return Err("camera must use x,y,z format, for example --camera 3,2,5".to_string());
    }
    let x = parts[0].parse().map_err(|_| "invalid camera x value")?;
    let y = parts[1].parse().map_err(|_| "invalid camera y value")?;
    let z = parts[2].parse().map_err(|_| "invalid camera z value")?;
    Ok(Vec3::new(x, y, z))
}

pub(crate) fn print_help() {
    eprintln!(
        "Usage: cargo run --release -- [options]\n\n\
         Options:\n\
           --scene <name>          sphere | cube-plane | all | all-alt (default: sphere)\n\
           --width <pixels>        Output width (default: 800)\n\
           --height <pixels>       Output height (default: 600)\n\
           --output, -o <file>     Write PPM to a file instead of stdout\n\
           --brightness <number>   Override light brightness for the selected scene\n\
           --camera <x,y,z>        Override camera position while keeping scene target\n\
           --reflections, -r       Enable reflective materials\n\
           --help, -h              Show this help\n\n\
         Example: cargo run --release -- --scene all --reflections > output.ppm"
    );
}
