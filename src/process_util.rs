use std::io::Read;
use sysinfo::{Pid, ProcessExt};
use crate::util::TMP_FILE;

pub fn get_reminder_processes(system: &mut impl sysinfo::SystemExt) -> Vec<(Pid, &sysinfo::Process)> {
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


pub fn list_alike(system: &mut impl sysinfo::SystemExt) {
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