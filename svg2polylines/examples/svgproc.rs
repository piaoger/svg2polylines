extern crate env_logger;
extern crate svg2polylines;
extern crate which;

use std::env;
use std::fs;
use std::io::Read;
use std::process::exit;

use svg2polylines::Polyline;

use std::ffi::OsStr;
use std::path::Path;

use std::path::PathBuf;
use std::process::Command;

fn path_to_string(path: PathBuf) -> Option<String> {
    match path.into_os_string().into_string() {
        Ok(x) => Some(x),
        Err(e) => None,
    }
}

fn add_path_env_var(s: &str) {
    if let Some(path) = env::var_os("PATH") {
        let mut paths = env::split_paths(&path).collect::<Vec<_>>();
        paths.push(PathBuf::from(s));
        let new_path = env::join_paths(paths).unwrap();
        env::set_var("PATH", &new_path);
    }
    print_env_var("PATH");
}

fn print_env_var(key: &str) {
    match env::var(key) {
        Ok(val) => println!("{}: {:?}", key, val),
        Err(e) => println!("couldn't interpret {}: {}", key, e),
    }
}

#[cfg(unix)]
fn dwgconvert() -> Command {
    match std::env::current_exe() {
        Err(_) => {
            println!("The process path could not be determined");
        }
        Ok(p) => {
            let parent = p.parent().unwrap().to_path_buf();
            match path_to_string(parent) {
                None => (),
                Some(s) => {
                    add_path_env_var(&s);
                }
            }
        }
    }

    match find_dwgconvert() {
        None => println!("cannot find dwgconvert.."),
        Some(path) => println!("{:?}", path),
    }

    Command::new("dwgconvert.sh")
}

#[cfg(windows)]
fn dwgconvert() -> Command {
    let mut cmd = Command::new("cmd.exe");
    cmd.args(&["/C", "dwgconvert"]);
    cmd
}

fn find_dwgconvert() -> Option<String> {
    match (which::which("dwgconvert.sh")) {
        Ok(pb) => path_to_string(pb),
        Err(e) => None,
    }
}

pub fn convert_dxf(input: &String, output: &String) {
    dwgconvert()
        .arg(&input)
        .arg(&output)
        .status()
        .ok()
        .expect("Failed to run \"dwgconvert\"!");
}

fn main() {
    // Logging
    env_logger::init();

    // Argument parsing
    let args: Vec<_> = env::args().collect();
    match args.len() {
        3 => {}
        _ => {
            println!("Usage: {} <path/to/file.svg>  <path/to/dir>", args[0]);
            exit(1);
        }
    };

    // Load file
    let input_pathstr = &args[1];
    let mut file = fs::File::open(&input_pathstr).unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();

    let output_dir = &args[2];

    let input_path = Path::new(&input_pathstr);
    let file_stem = input_path
        .file_stem()
        .and_then(|os| os.to_str())
        .unwrap_or("output");

    let ext = input_path
        .extension()
        .and_then(|os| os.to_str())
        .unwrap_or("");

    let polylines: Vec<Polyline> = match ext {
        "svg" => svg2polylines::parse_svg(&s),
        "dxf" => svg2polylines::parse_dxf(&s),
        _ => {
            // some default
            vec![]
        }
    };

    // Parse data

    let simplifyvec = svg2polylines::simplify(&polylines);

    if Path::new("/tmp/svgproc").exists() {
        fs::remove_dir_all("/tmp/svgproc").unwrap();
    }

    fs::create_dir_all("/tmp/svgproc").unwrap();

    if !Path::new(&output_dir).exists() {
        fs::create_dir_all(&output_dir).unwrap();
    }

    svg2polylines::write_svg(
        &simplifyvec,
        output_dir.to_owned() + "/" + &file_stem.to_owned() + ".svg",
    );
    svg2polylines::write_dxf(
        &simplifyvec,
        "/tmp/svgproc/".to_owned() + &file_stem.to_owned() + ".dxf",
    );
    //svg2polylines::write_svg(&polylines, output_dir.to_owned() + "/"+ &file_stem.to_owned()+"-rewrite.svg");

    convert_dxf(
        &("/tmp/svgproc/".to_owned() + &file_stem.to_owned() + ".dxf"),
        &output_dir,
    );

    // Print data
    let mut sum = 0usize;
    for p in simplifyvec.iter() {
        sum += p.len();
    }
    println!("polylines:{}, points: {}", simplifyvec.len(), sum);

    // Print data
    println!("Found {} polylines.", simplifyvec.len());
    for line in simplifyvec {
        //println!("- {:?}", line);
    }
}
