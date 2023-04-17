#[macro_use]
extern crate clap;
extern crate serde_json;
extern crate serde;

use clap::{Arg, App};
use std::env;
use std::thread;
use std::process::Command;
use std::time::Duration;
use std::io::{self, Write}; //, IsTerminal}; #![feature(is_terminal)]
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Loudness {
    input_i: String,
    input_tp: String,
    input_lra: String,
    input_thresh:String,
    target_offset: String,
}

fn progress_thread() -> Arc<AtomicBool> {
    const PROGRESS_CHARS: [&str; 12] = ["⠂", "⠃", "⠁", "⠉", "⠈", "⠘", "⠐", "⠰", "⠠", "⠤", "⠄", "⠆"];
    let finished = Arc::new(AtomicBool::new(false));
    //if io::stderr().is_terminal() { //            TODO: uncomment when stabilizes
        let stop_signal = Arc::clone(&finished);
        let _ = thread::spawn(move || {
            for pc in PROGRESS_CHARS.iter().cycle() {
                if stop_signal.load(Ordering::Relaxed) {
                    break;
                };
                write!(io::stderr(), "Processing {}\r", pc).unwrap();
                thread::sleep(Duration::from_millis(250));
            }
        });
    //}
    finished
}

fn main() {
    let matches = App::new("ffmpeg-loudnorm-helper")
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
        .arg(Arg::with_name("INPUT")
            .help("Sets the input file to scan")
            .required(true)
            )
        .arg(Arg::with_name("I")
            .short("i")
            .long("i")
            .required(false)
            .allow_hyphen_values(true)
            .default_value("-16.0")
            .help("Integrated loudness target. Range is -70.0 - -5.0. Default value is -16.0"))
        .arg(Arg::with_name("LRA")
            .short("l")
            .long("lra")
            .required(false)
            .default_value("6.0")
            .help("Loudness range target. Range is 1.0 - 20.0. Default value is 6.0"))
        .arg(Arg::with_name("TP")
            .short("t")
            .long("tp")
            .required(false)
            .allow_hyphen_values(true)
            .default_value("-1.0")
            .help("Maximum true peak. Range is -9.0 - +0.0. Default value is -1.0"))
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap();
    let target_i : f32 = matches.value_of("I").unwrap().parse().unwrap();
    let target_lra : f32 = matches.value_of("LRA").unwrap().parse().unwrap();
    let target_tp : f32 = matches.value_of("TP").unwrap().parse().unwrap();

    let mut command = Command::new("ffmpeg");
    command.current_dir(&env::current_dir().unwrap())
        .arg("-i")
        .arg(&input_path)
        .arg("-hide_banner")
        .args(&["-vn", "-af"])
        .arg(format!("loudnorm=I={}:LRA={}:tp={}:print_format=json",target_i,target_lra,target_tp))
        .args(&["-f", "null", "-"]);

    let output = {
        let finished = progress_thread();
        let output_res = command.output();
        finished.store(false, Ordering::Relaxed);
        output_res.expect("Failed to execute ffmpeg process")
    };

    let output_s = String::from_utf8_lossy(&output.stderr);
    let lines: Vec<&str> = output_s.lines().collect();
    let (_, lines) = lines.split_at(lines.len() - 12);
    let json: String = lines.join("\n");

    let loudness: Loudness = serde_json::from_str(&json).unwrap();
    let af = format!("-af loudnorm=linear=true:I={}:LRA={}:TP={}:measured_I={}:measured_TP={}:measured_LRA={}:measured_thresh={}:offset={}:print_format=summary",
            target_i, target_lra, target_tp,
            loudness.input_i,
            loudness.input_tp,
            loudness.input_lra,
            loudness.input_thresh,
            loudness.target_offset
    );

    print!("{}", af);
    //io::stdout().write_all(&output.stderr).unwrap();
}
