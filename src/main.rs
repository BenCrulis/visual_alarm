mod display;
mod util;
mod process_util;

use std::error::Error;
use std::time::Instant;

use std::fs::File;

use clap::{App, Arg};
use time;
use chrono;
use fork;

use sysinfo::{SystemExt, ProcessExt, Pid};
use std::fmt::Debug;
use std::io::{Write, Read};


use crate::util::{TMP_FILE, find_visual_with_depth, set_remainder};
use crate::process_util::{get_reminder_processes, list_alike};



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
