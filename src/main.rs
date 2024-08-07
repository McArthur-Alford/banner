use clap::Parser;
use figlet_rs::FIGfont;
use noise::{NoiseFn, Perlin};
use palette::{Gradient, LinSrgb};
use rand::rngs::StdRng;
use rand::{random, Rng, SeedableRng};
use std::io::{self, Write};
use termion::terminal_size;

/// Generate an ASCII heatmap with Perlin noise
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of rows
    rows: usize,

    /// Scale
    #[arg(short, long, default_value_t = 100.0)]
    scale: f64,

    /// Octaves
    #[arg(short, long, default_value_t = 6)]
    octaves: usize,

    /// Persistence
    #[arg(short, long, default_value_t = 0.5)]
    persistence: f64,

    /// Lacunarity
    #[arg(short, long, default_value_t = 2.0)]
    lacunarity: f64,

    /// Fade factor range
    #[arg(short, long, default_value_t = 0.1)]
    fade_factor_range: f64,

    /// Random seed
    #[arg(short, long)]
    random: Option<u64>,

    /// Text to overlay
    #[arg(short, long)]
    text: String,
}

fn main() {
    let args = Args::parse();

    let (cols, _) = terminal_size().unwrap_or((80, 20));
    let cols = cols as usize;

    let mut rng = StdRng::seed_from_u64(args.random.unwrap_or(random()));
    let perlin = Perlin::new();

    let mut data = vec![vec![0.0; cols]; args.rows];
    for i in 0..args.rows {
        for j in 0..cols {
            let mut val = 0.0;
            let mut frequency = 1.0;
            let mut amplitude = 1.0;
            let mut max_value = 0.0;
            for _ in 0..args.octaves {
                val += perlin.get([
                    i as f64 / args.scale * frequency,
                    j as f64 / args.scale * frequency,
                ]) * amplitude;
                max_value += amplitude;
                amplitude *= args.persistence;
                frequency *= args.lacunarity;
            }
            data[i][j] = val / max_value;
        }
    }

    let min_val = data
        .iter()
        .flat_map(|row| row.iter())
        .cloned()
        .fold(f64::INFINITY, f64::min);
    let max_val = data
        .iter()
        .flat_map(|row| row.iter())
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);

    for row in data.iter_mut() {
        for val in row.iter_mut() {
            *val = (*val - min_val) / (max_val - min_val);
        }
    }

    let gradient = Gradient::new(vec![
        LinSrgb::new(0.157, 0.165, 0.212), // Final color #282A36
        LinSrgb::new(0.0, 0.5, 0.7),       // Adjusted light blue
        LinSrgb::new(0.545, 0.914, 0.992), // #8BE9FD (cyan from Dracula theme)
        LinSrgb::new(0.7, 0.85, 0.9),      // Adjusted light cyan
    ]);

    let chars = ["█"];
    let mut stdout = io::stdout();

    // Generate heatmap
    let mut heatmap = vec![vec![(0.0, "█"); cols]; args.rows];
    for (i, row) in data.iter().enumerate() {
        for (j, &val) in row.iter().enumerate() {
            let fade_factor = 1.0
                - (j as f64 / cols as f64)
                    * (1.0 + rng.gen_range(-args.fade_factor_range..=args.fade_factor_range));
            let fade_factor = fade_factor.clamp(0.0, 1.0);
            // let val = val * fade_factor;

            let char_index = (val * (chars.len() - 1) as f64).round() as usize;
            let ch = chars[char_index];

            let color = gradient.get(val);
            let (r, g, b) = (
                (color.red * 255.0) as u8,
                (color.green * 255.0) as u8,
                (color.blue * 255.0) as u8,
            );

            heatmap[i][j] = (val, ch);
            // write!(stdout, "\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, ch).unwrap();
        }
        // writeln!(stdout).unwrap();
    }

    // Overlay figlet text with stretched background
    let standard_font = FIGfont::from_content(include_str!("../font.flf")).unwrap();
    let figure = standard_font.convert(&args.text).unwrap();
    let string = figure.to_string();
    let figlet_lines: Vec<&str> = string.lines().collect();
    let figlet_height = figlet_lines.len();

    for (i, &line) in figlet_lines.iter().enumerate() {
        if i >= figlet_height - 2 {
            continue;
        }
        for j in 0..cols {
            if i < args.rows {
                let (val, _) = heatmap[i][j];
                let color = gradient.get(val);
                let (mut r, mut g, mut b) = (
                    (color.red * 255.0) as u8,
                    (color.green * 255.0) as u8,
                    (color.blue * 255.0) as u8,
                );

                let ch = if j < line.len() {
                    line.chars().nth(j).unwrap_or(' ')
                } else {
                    ' '
                };

                // write!(stdout, "\x1b[48;2;{};{};{}m{}\x1b[0m", r, g, b, ch).unwrap();
                // Calculate the luminance of the background color
                let luminance = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;

                // Determine the text color based on the luminance
                let (text_r, text_g, text_b) = if luminance > 128.0 {
                    (0, 0, 0) // Bright background, use black text
                } else {
                    (255, 255, 255) // Dark background, use white text
                };

                if ch != ' ' {
                    r = (0.4 * r as f32).round() as u8;
                    g = (0.4 * g as f32).round() as u8;
                    b = (0.4 * b as f32).round() as u8;
                }

                // let ch = if ch == ' ' { ' ' } else { '⠿' };
                // let ch = if ch == ' ' { ' ' } else { '█' };
                let ch = if ch == ' ' { ' ' } else { ' ' };

                write!(
                    stdout,
                    "\x1b[48;2;{};{};{}m\x1b[38;2;{};{};{}m{}\x1b[0m",
                    r,
                    g,
                    b, // Background color
                    text_r,
                    text_g,
                    text_b, // Text color
                    ch      // Character to print
                )
                .unwrap();
            }
        }
        writeln!(stdout).unwrap();
    }
}
