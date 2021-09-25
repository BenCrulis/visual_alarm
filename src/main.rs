mod display;
mod util;

use std::error::Error;
use std::time::Instant;

use std::fs::File;

use clap::{App, Arg};
use time;
use chrono;
use fork;

use crate::util::find_visual_with_depth;
use sysinfo::{SystemExt, ProcessExt, Pid};
use std::fmt::Debug;
use std::io::{Write, Read};

const TMP_FILE: &str = "/tmp/visual_alarm_description";


fn set_remainder(nb_seconds: u64) -> Result<(), Box<dyn Error>> {
    std::thread::sleep(std::time::Duration::from_secs(nb_seconds));

    let mut display_obj = display::Display::create_and_connect()?;
    display_obj.default_screen_pulse_effect();
    Ok(())
}


fn get_reminder_processes(system: &mut impl sysinfo::SystemExt) -> Vec<(Pid, &sysinfo::Process)> {
    let own_pid = sysinfo::get_current_pid().unwrap();
    //let mut system = sysinfo::System::new_all();
    //system.refresh_system();

    //println!("own process: {}", own_pid);

    let exe_name = "visual_alarm";

    system.processes().iter().filter_map(|(&pid, process)| {
        if process.name() == exe_name && pid != own_pid {
            Some((pid, process))
        }
        else {
            None
        }
    }).collect()
}


fn list_alike(system: &mut impl sysinfo::SystemExt) {
    for (pid, _) in get_reminder_processes(system) {
        //print!("{}: {}", pid, process.name());
        //println!("{:?}", process.cmd());

        for entry in std::fs::read_dir(format!("/proc/{}/fd/", pid)).unwrap() {
            let entry = entry.unwrap();
            let entry_name = entry.file_name();
            let fd = entry_name.to_str().unwrap();
            let link = std::fs::read_link(format!("/proc/{}/fd/{}", pid, fd)).unwrap();

            let link_name = link.to_str().unwrap();
            if link_name.starts_with(TMP_FILE) {
                let path = entry.path();
                let mut file = std::fs::OpenOptions::new().read(true).open(&path).unwrap();
                //let link = std::fs::read_link(&path).unwrap();
                //println!("link: {}", link.to_str().unwrap());
                //std::fs::remove_file(filepath).unwrap();
                let mut buffer = String::new();
                file.read_to_string(&mut buffer);
                //println!("content of file: {}", buffer);
                println!("{}: {}", &pid, buffer);
                break
            }
        }
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("Visual Alarm")
        .version("0.0.1")
        .author("Ben Crulis")
        .about("Set a visual reminder at given time\nExample: visual_alarm 5  # sets a reminder for now+5 minutes")
        .arg(Arg::new("NB_MINUTES")
                .required(false)
                .default_value("10")
                .number_of_values(1))
        .arg(Arg::new("nb_pulses")
                .long("pulses")
                .default_value("5")
                .number_of_values(1))
        .arg(Arg::new("no_child")
            .long("no-child")
            .required(false)
            )
        .subcommand(App::new("killall").about("kill all reminder processes"))
        .subcommand(App::new("list").about("list current reminders"))
        .get_matches();

    let mut system = sysinfo::System::new_all();
    system.refresh_system();

    if matches.subcommand_matches("killall").is_some() {
        println!("killall used");
        let mut i = 0;
        for (_, process) in get_reminder_processes(&mut system) {
            process.kill(sysinfo::Signal::Term);
            i += 1;
        }
        println!("killed {} process{}", i, if i > 1 {"es"} else {""});
        return Ok(());
    }
    else if matches.subcommand_matches("list").is_some() {
        list_alike(&mut system);
        return Ok(());
    }

    let nb_minutes_str = matches.value_of("NB_MINUTES").unwrap();

    let no_child = matches.is_present("no_child");

    if let Some(nb_minutes) = nb_minutes_str.parse::<u16>().ok() {
        let mut file = std::fs::OpenOptions::new().create_new(true).write(true).truncate(true).open(TMP_FILE).unwrap();
        std::fs::remove_file(TMP_FILE).unwrap();

        let nb_seconds = nb_minutes as u64 * 60;

        let mut call_time = chrono::Local::now();

        let remind_time = (call_time + chrono::FixedOffset::east(nb_seconds as i32));

        println!("Reminder set for now+{} minutes: {}", nb_minutes, remind_time);

        file.write_all(format!("set for {}", remind_time).as_bytes());
        file.flush();
        //println!("written to file");

        if no_child {
            set_remainder(nb_seconds)?;
        }
        else {
            if let Ok(fork::Fork::Child) = fork::daemon(false, true) {
                if let Err(e) = set_remainder(nb_seconds) {
                    println!("Error in child process!");
                    return Err(e);
                }
            }
            else {

            }
        }
    }
    else {
        println!("incorrect number of minutes: \"{}\"\nexiting...", nb_minutes_str);
    }

    Ok(())

}