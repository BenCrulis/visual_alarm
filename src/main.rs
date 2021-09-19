mod display;
mod util;

use std::error::Error;
use std::time::Instant;

use clap::{App, Arg};
use time;
use chrono;
use fork;

use crate::util::find_visual_with_depth;


fn set_remainder(nb_seconds: u64) -> Result<(), Box<dyn Error>> {
    std::thread::sleep(std::time::Duration::from_secs(nb_seconds));

    let mut display_obj = display::Display::create_and_connect()?;
    display_obj.default_screen_pulse_effect();
    Ok(())
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("Visual Alarm")
        .version("0.0.1")
        .author("Ben Crulis")
        .about("Set a visual reminder at given time")
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
        .get_matches();

    let nb_minutes_str = matches.value_of("NB_MINUTES").unwrap();

    let no_child = matches.is_present("no_child");

    if let Some(nb_minutes) = nb_minutes_str.parse::<u16>().ok() {
        let nb_seconds = nb_minutes as u64 * 60;

        let mut call_time = chrono::Local::now();

        let remind_time = (call_time + chrono::FixedOffset::east(nb_seconds as i32));

        println!("Reminder set for now+{} minutes: {}", nb_minutes, remind_time);

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