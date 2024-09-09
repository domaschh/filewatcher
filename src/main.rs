use nix::sys::event::{EventFilter, EventFlag, FilterFlag, KEvent, Kqueue};
use std::env;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::io::AsRawFd;
use std::path::Path;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <file_to_monitor>", args[0]);
        std::process::exit(1);
    }
    let file_path = Path::new(&args[1]);
    let dir_path = file_path.parent().unwrap_or(Path::new("/"));
    let file_name = file_path.file_name().unwrap_or(OsStr::new(""));

    println!(
        "Monitoring directory: {:?} for changes to file: {:?}",
        dir_path, file_name
    );

    let kq = Kqueue::new()?;

    let dir = File::open(dir_path)?;
    let fd = dir.as_raw_fd();

    loop {
        let change_event = KEvent::new(
            fd as usize,
            EventFilter::EVFILT_VNODE,
            EventFlag::EV_ADD | EventFlag::EV_CLEAR | EventFlag::EV_ENABLE,
            FilterFlag::NOTE_WRITE
                | FilterFlag::NOTE_EXTEND
                | FilterFlag::NOTE_ATTRIB
                | FilterFlag::NOTE_RENAME
                | FilterFlag::NOTE_DELETE,
            0,
            0,
        );
        let mut event_list = vec![KEvent::new(
            0,
            EventFilter::EVFILT_VNODE,
            EventFlag::empty(),
            FilterFlag::empty(),
            0,
            0,
        )];

        match kq.kevent(&[change_event], &mut event_list, None) {
            Ok(events) if events > 0 => {
                if let Ok(metadata) = std::fs::metadata(file_path) {
                    if metadata.is_file() {
                        if let Err(e) = write_custom_config(file_path) {
                            panic!("Couldn't write custom config to file {}", e);
                        }
                    }
                }
            }
            Ok(_) => {
                println!("No events received")
            }
            Err(e) => {
                eprintln!("Error in kevent: {:?}", e);
                break;
            }
        }
    }
    Ok(())
}

fn write_custom_config(file_path: &Path) -> Result<(), std::io::Error> {
    let custom_config = include_bytes!("../customconfig");
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(file_path)?;
    file.write_all(custom_config)?;
    Ok(())
}
