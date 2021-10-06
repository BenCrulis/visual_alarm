mod display;
mod util;
mod process_util;

use std::error::Error;

use clap::{App, Arg};
use chrono;
use fork;

use sysinfo::{SystemExt, ProcessExt};
use std::io::{Write};


use crate::util::{TMP_FILE, fire_remainder};
use crate::process_util::{get_reminder_processes, list_alike, run_function, try_notify_send};


fn main() -> Result<(), Box<dyn Error>> {
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
    let nb_pulses_str = matches.value_of("nb_pulses").unwrap();
    let nb_pulses = {
        if let Ok(nb_pulses) = nb_pulses_str.parse::<u8>() {
            nb_pulses
        }
        else {
            eprintln!("Number of pulses is not an integer or has invalid range");
            std::process::exit(1);
        }
    };

    let no_child = matches.is_present("no_child");

    if let Some(nb_minutes) = nb_minutes_str.parse::<u16>().ok() {
        let mut file = std::fs::OpenOptions::new().create_new(true).write(true).truncate(true).open(TMP_FILE).unwrap();
        std::fs::remove_file(TMP_FILE).unwrap();

        let nb_seconds = nb_minutes as u64 * 60;

        let call_time = chrono::Local::now();

        let remind_time = call_time + chrono::FixedOffset::east(nb_seconds as i32);

        println!("Reminder set for now+{} minutes: {}", nb_minutes, remind_time);

        file.write_all(format!("set for {}", remind_time).as_bytes()).unwrap();
        file.flush().unwrap();
        //println!("written to file");

        return run_function(|| {
            std::thread::sleep(std::time::Duration::from_secs(nb_seconds));
            if let Err(err) = try_notify_send(format!("Reminder of {} minutes", nb_minutes)) {
                eprintln!("Could not call notify-send: {}", err);
            }
            let res = fire_remainder(nb_pulses);

            return res;
        }, !no_child);

    }
    else {
        println!("incorrect number of minutes: \"{}\"\nexiting...", nb_minutes_str);
    }

    Ok(())

}
