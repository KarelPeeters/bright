use std::process::Command;

use ddc::Ddc;
use ddc_winapi::Monitor;
use clap::Clap;
use std::num::ParseIntError;


#[derive(Clap)]
struct Opts {
    internal: String,
    external: Option<String>,
}

fn find_guid<'a>(string: &'a str, query: &str) -> Option<&'a str> {
    string.lines().find_map(|line| {
        if line.contains(query) {
            let guid = &line[line.find(": ")? + 2..line.rfind("  (")?];
            assert_eq!(guid.len(), 36);
            Some(guid)
        } else {
            None
        }
    })
}

fn set_brightness(internal: u8, external: u8) {
    assert!(internal <= 100);
    assert!(external <= 100);

    //powercfg
    let power_query_output =
        Command::new("powercfg")
            .args(&["/q"])
            .output()
            .expect("failed to run powercfg query");

    let power_query =
        std::str::from_utf8(&power_query_output.stdout)
            .expect("Invalid output from powercfg");

    let scheme_guid = find_guid(power_query, "Power Scheme GUID").unwrap();
    let subgroup_guid = find_guid(power_query, "(Display)").unwrap();
    let setting_guid = find_guid(power_query, "(Display brightness)").unwrap();

    let value = internal.to_string();

    Command::new("powercfg")
        .args(&["-SetDcValueIndex", &scheme_guid, &subgroup_guid, &setting_guid, &value])
        .output()
        .expect("Failure setting AC brightness value");

    Command::new("powercfg")
        .args(&["-SetAcValueIndex", &scheme_guid, &subgroup_guid, &setting_guid, &value])
        .output()
        .expect("Failure setting AC brightness value");

    let monitors = Monitor::enumerate().unwrap();

    for mut mon in monitors {
        //ignore errors
        //TODO call the correct functions instead of calling the wrong one and then ignoring the error
        let _ = mon.set_vcp_feature(0x10, external as u16);
        let _ = mon.save_current_settings();
    }

    Command::new("powercfg")
        .args(&["-S", &scheme_guid])
        .output()
        .expect("failed to apply updated power scheme");
}

fn parse_brightness_string(s: &str) -> Result<u8, ParseIntError> {
    let value = s.parse::<u8>()?;

    if s.len() == 1 {
        Ok(value * 10)
    } else {
        Ok(value)
    }
}

fn main_inner(opts: Opts) -> Result<(), ParseIntError> {
    let internal = parse_brightness_string(&opts.internal)?;
    let external = opts.external.as_ref().unwrap_or(&opts.internal);
    let external = parse_brightness_string(&external)?;

    if internal <= 100 && external <= 100 {
        println!("Internal: {}, External: {}", internal, external);
        set_brightness(internal, external);
    } else {
        eprintln!("Brightness must be <= 100");
    }

    Ok(())
}

fn main() {
    let opts: Opts = Opts::parse();

    if let Err(err) = main_inner(opts) {
        eprintln!("Error: {}", err);
    }
}
