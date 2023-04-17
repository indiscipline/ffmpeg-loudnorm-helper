#[macro_use]
extern crate clap;
extern crate serde_json;
extern crate serde;

use clap::Arg;
use std::env;
use std::thread;
use std::process::Command;
use std::time::Duration;
//use std::io::{self, Write, IsTerminal}; #![feature(is_terminal)]
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use serde::{Serialize, Deserialize};
use std::f32;

#[derive(Serialize, Deserialize, Debug)]
struct Loudness {
    input_i: String,
    input_tp: String,
    input_lra: String,
    input_thresh: String,
    target_offset: String,
}

fn progress_thread() -> (Arc<AtomicBool>, thread::JoinHandle<()>) {
    const PROGRESS_CHARS: [&str; 12] = ["⠂", "⠃", "⠁", "⠉", "⠈", "⠘", "⠐", "⠰", "⠠", "⠤", "⠄", "⠆"];
    let finished = Arc::new(AtomicBool::new(false));
    //if io::stderr().is_terminal() { //            TODO: uncomment when stabilizes
    let stop_signal = Arc::clone(&finished);
    let handle = thread::spawn(move || {
        for pc in PROGRESS_CHARS.iter().cycle() {
            if stop_signal.load(Ordering::Relaxed) {
                break;
            };
            eprint!("Processing {}\r", pc);
            thread::sleep(Duration::from_millis(250));
        }
    });
    //}
    (finished, handle)
}


fn main() {
    let matches = clap::builder::Command::new("ffmpeg-loudnorm-helper")
        .version(crate_version!())
        .author(crate_authors!())
        .about(
"Helper for linear audio loudness normalization using ffmpeg's loudnorm filter.
Performs the loudness scanning pass of the given file and outputs the string
of desired loudnorm options to be included in ffmpeg arguments.

Designed to work using your shell's command substitution. Bash example:
    'ffmpeg -i input.mov -c:v copy -c:a libopus $(ffmpeg-lh input.mov) normalized.mkv'
Windows CMD:
    'for /f \"tokens=*\" %i in ('ffmpeg-lh input.mov') do ffmpeg -i input.mov -c:v copy -c:a libopus %i normalized.mkv'"
        )
        .arg(Arg::new("INPUT")
            .help("Sets the input file to scan")
            .required(true)
            )
        .arg(Arg::new("I")
            .short('i')
            .long("i")
            .ignore_case(true)
            .required(false)
            .allow_hyphen_values(true)
            .default_value("-18.0")
            .help("Integrated loudness target. Clamped to valid range [-70.0..-5.0]."))
        .arg(Arg::new("LRA")
            .short('l')
            .long("lra")
            .ignore_case(true)
            .required(false)
            .default_value("12.0")
            .help("Loudness range target. Clamped to valid range [1.0..20.0]."))
        .arg(Arg::new("TP")
            .short('t')
            .long("tp")
            .ignore_case(true)
            .required(false)
            .allow_hyphen_values(true)
            .default_value("-1.0")
            .help("Maximum true peak. Clamped to valid range [-9.0..0.0]."))
        .arg(Arg::new("resample")
            .short('r')
            .long("resample")
            .action(clap::ArgAction::SetTrue)
            .help("Add a resampling filter hardcoded to 48kHz after loudnorm (which might upsample to 192kHz)"))
        .get_matches();

    let input_path = matches.get_one::<String>("INPUT").unwrap(); // defaults provided = safe
    let target_i = matches.get_one::<String>("I").unwrap().parse::<f32>().unwrap().clamp(-70.0, -5.0);
    let target_lra = matches.get_one::<String>("LRA").unwrap().parse::<f32>().unwrap().clamp(1.0, 20.0);
    let target_tp = matches.get_one::<String>("TP").unwrap().parse::<f32>().unwrap().clamp(-9.0, 0.0);

    let mut command = Command::new("ffmpeg");
    command.current_dir(&env::current_dir().unwrap())
        .arg("-i")
        .arg(&input_path)
        .arg("-hide_banner")
        .args(&["-vn", "-af"])
        .arg(format!("loudnorm=I={}:LRA={}:tp={}:print_format=json",target_i,target_lra,target_tp))
        .args(&["-f", "null", "-"]);

    let output = {
        let (finished, _) = progress_thread();
        let output_res = command.output();
        finished.store(false, Ordering::SeqCst);
        output_res.expect("Failed to execute ffmpeg process!")
    };

    let output_s = String::from_utf8_lossy(&output.stderr);
    if output.status.success() {
        let loudness: Loudness = {
            let json: String = {
                let lines: Vec<&str> = output_s.lines().collect();
                let (_, lines) = lines.split_at(lines.len() - 12);
                lines.join("\n")
            };
            serde_json::from_str(&json).unwrap()
        };

        {
            let measured_tp = loudness.input_tp.parse::<f32>().expect("Measured TP value is not a valid number!");
            let measured_i =  loudness.input_i.parse::<f32>().expect("Measured TP value is not a valid number!");
            let tp_diff = target_tp - measured_tp;
            let i_diff = target_i - measured_i;
            if i_diff > tp_diff {
                eprintln!("⚠ Not enough headroom! Dynamic normalization will be used. Headroom: {}dB, required: {}dB.", tp_diff, i_diff);
            }
        };

        let af = format!("-af loudnorm=linear=true:I={}:LRA={}:TP={}:measured_I={}:measured_TP={}:measured_LRA={}:measured_thresh={}:offset={}:print_format=summary{}",
                target_i, target_lra, target_tp,
                loudness.input_i,
                loudness.input_tp,
                loudness.input_lra,
                loudness.input_thresh,
                loudness.target_offset,
                if matches.get_flag("resample") {
                        ",aresample=osr=48000,aresample=resampler=soxr:precision=28"
                    } else { "" }
        );
        print!("{}", af);
    } else {
        eprintln!("{}", output_s);
    }
}
