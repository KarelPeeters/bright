use std::process::Command;

use ddc::Ddc;
use ddc_winapi::Monitor;
use clap::Clap;


#[derive(Clap)]
struct Opts {
    internal: u8,
    external: Option<u8>,
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
        .expect("Failure setting AC brigtness value");

    Command::new("powercfg")
        .args(&["-SetAcValueIndex", &scheme_guid, &subgroup_guid, &setting_guid, &value])
        .output()
        .expect("Failure setting AC brigtness value");

    let monitors = Monitor::enumerate().unwrap();

    for mut mon in monitors {
        println!("{:?}", mon.set_vcp_feature(0x10, external as u16));
        println!("{:?}", mon.save_current_settings());
    }

    Command::new("powercfg")
        .args(&["-S", &scheme_guid])
        .output()
        .expect("failed to apply updated power scheme");
}

fn main() {
    let opts: Opts = Opts::parse();

    let external = opts.external.unwrap_or(opts.internal);

    if opts.internal <= 100 && external <= 100 {
        set_brightness(opts.internal, external);
    }else {
        eprintln!("Brightness must be <= 100")
    }
}
