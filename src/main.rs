use chrono::{NaiveTime, Timelike, Duration};
use std::thread;
use std::time::Duration as StdDuration;
use home::home_dir;
use rodio::{Decoder, OutputStream, Sink};
use std::error::Error;
use std::fs;
use serde::{Deserialize, Serialize};
extern crate serde_json;
use rand::seq::SliceRandom;
use std::path::Path;
use std::process::Command;

#[derive(Debug)]
struct TimeInfo {
    time: NaiveTime,
    info: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct PrayerTimesResponse {
    fajr: String,
    dhuhr: String,
    asr: String,
    magrib: String,
    isha: String,
}



fn main() {    
    if let Err(err) = create_adhanapp_folders() {
        eprintln!("Error: {}", err);
    }

    // Fetch and process prayer times when the program starts
    if let Err(err) = fetch_and_process_prayer_times() {
        eprintln!("Error fetching and processing prayer times: {}", err);
        // Terminate the program if fetching and processing fails
        return;
    }

    // Spawn a separate thread to continuously update prayer times at 1:00 AM
    let _ = thread::spawn(|| {
        loop {
            // Calculate the duration until 1:00 AM
            let current_time = chrono::Local::now().time();
            let end_time = match NaiveTime::from_hms_opt(1, 0, 0) {
                Some(time) => time,
                None => {
                    eprintln!("Invalid time specified.");
                    return;
                }
            };
            let duration_until_1_am = calculate_duration(&current_time, &end_time);
            println!("Duration until next fetch time of prayer times: {}", format_duration(duration_until_1_am));

            // Sleep until 1:00 AM
            thread::sleep(duration_until_1_am.to_std().unwrap());
            

            // Fetch and process prayer times
            if let Err(err) = fetch_and_process_prayer_times() {
                eprintln!("Error fetching and processing prayer times: {}", err);
            }
        }
    });
  
    
    loop {
        
        
        let prayer_times = fetch_prayer_times_from_file();
        

        match prayer_times {
            Ok(times) => {
                println!("Prayer times: {:?}", times);
                let times: Vec<TimeInfo> = vec![
                    TimeInfo {
                        time: NaiveTime::parse_from_str(&times.fajr, "%H:%M").unwrap(),
                        info: String::from("Fajr"),
                    },
                    TimeInfo {
                        time: NaiveTime::parse_from_str(&times.dhuhr, "%H:%M").unwrap(),
                        info: String::from("Dhuhr"),
                    },
                    TimeInfo {
                        time: NaiveTime::parse_from_str(&times.asr, "%H:%M").unwrap(),
                        info: String::from("Asr"),
                    },
                    TimeInfo {
                        time: NaiveTime::parse_from_str(&times.magrib, "%H:%M").unwrap(),
                        info: String::from("Magrib"),
                    },
                    TimeInfo {
                        time: NaiveTime::parse_from_str(&times.isha, "%H:%M").unwrap(),
                        info: String::from("Isha"),
                    },
                ];            
                let current_time = chrono::Local::now().time();
                let upcoming_time = find_upcoming_time(&times, &current_time);
                match upcoming_time {
                    Some(time_info) => {
                        println!("Next time is {} at {}", time_info.info, time_info.time);
                        let duration_until_next = calculate_duration(&current_time, &time_info.time);
                        println!("Duration until next time: {}", format_duration(duration_until_next));

                        let duration_seconds = duration_until_next.num_seconds();
                        //let duration_seconds = 10; // for testing purposes
                        
                        thread::sleep(StdDuration::from_secs(duration_seconds as u64));
                        println!("Time is up! Proceeding now.");

                        set_rhythmbox_volume(0); // Turn volume off
                        set_mpd_volume(0); // Set MPD volume to 0
                    
                        play_adhan(&time_info.info).unwrap();

                        set_rhythmbox_volume(1); // Turn volume on
                        set_mpd_volume(70); // Restore MPD volume to 70
                        
                    }
                    None => {
                        println!("No upcoming time found. Exiting loop.");
                        break; // Break out of the loop if no upcoming time is found
                    }
                }
            },
            Err(e) => {
                println!("Error fetching prayer times: {:?}", e);
            }
        }
    }
}

fn set_rhythmbox_volume(volume: u8) {
    let rhythmbox_client_path = "/usr/bin/rhythmbox-client";
    if Path::new(rhythmbox_client_path).exists() {
        println!("Setting rhythmbox client volume to {}", volume);
        Command::new(rhythmbox_client_path)
            .arg("--set-volume")
            .arg(volume.to_string())
            .output()
            .expect("Failed to execute rhythmbox-client command");
    }
}

fn set_mpd_volume(volume: u8) {
    let mpc_path = "/usr/bin/mpc";
    if Path::new(mpc_path).exists() {
        println!("Setting MPD volume to {}", volume);
        Command::new(mpc_path)
            .arg("volume")
            .arg(volume.to_string())
            .output()
            .expect("Failed to execute mpc volume");
    }
}

fn fetch_and_process_prayer_times() -> Result<(), Box<dyn Error>> {
    let prayer_times = fetch_prayer_times_from_api()?;
    save_prayer_times_to_file(&prayer_times)?;
    Ok(())
}

fn save_prayer_times_to_file(prayer_times: &PrayerTimesResponse) -> Result<(), Box<dyn Error>> {
    let prayer_times_json = serde_json::to_string(prayer_times)?;
    let home_dir = home_dir().ok_or(std::io::Error::new(std::io::ErrorKind::Other, "Failed to determine home directory"))?;
    let file_path = home_dir.join("adhanapp/prayer_times.json");
    fs::write(&file_path, prayer_times_json)?;
    Ok(())
}

fn fetch_prayer_times_from_file() -> Result<PrayerTimesResponse, Box<dyn Error>> {
    let home_dir = home_dir().ok_or(std::io::Error::new(std::io::ErrorKind::Other, "Failed to determine home directory"))?;
    let file_path = home_dir.join("adhanapp/prayer_times.json");
    let prayer_times_json = fs::read_to_string(&file_path)?;
    let prayer_times: PrayerTimesResponse = serde_json::from_str(&prayer_times_json)?;
    Ok(prayer_times)
}

fn create_adhanapp_folders() -> Result<(), Box<dyn Error>> {
    let home_dir = home_dir().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::Other, "Failed to determine home directory")
    })?;

    let adhanapp_media_dir = home_dir.join("adhanapp/media");
    let fajr_dir = adhanapp_media_dir.join("fajr");
    let other_dir = adhanapp_media_dir.join("other");

    create_directory(&adhanapp_media_dir)?;
    create_directory(&fajr_dir)?;
    create_directory(&other_dir)?;

    Ok(())
}

fn create_directory(path: &Path) -> Result<(), Box<dyn Error>> {
    if !path.exists() {
        fs::create_dir_all(path)?;
        println!("Folder '{}' created successfully!", path.display());
    }
    Ok(())
}

fn play_adhan(prayer_name: &str) -> Result<(), Box<dyn Error>> {
    let home_dir = home_dir().ok_or(std::io::Error::new(std::io::ErrorKind::Other, "Failed to determine home directory"))?;
    let folder_name = if prayer_name == "Fajr" {
        "fajr"
    } else {
        "other"
    };
    let audio_files_path = home_dir.join("adhanapp/media").join(folder_name);

    let audio_files: Vec<_> = std::fs::read_dir(audio_files_path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file() && entry.path().extension().map_or(false, |ext| ext == "mp3"))
        .map(|entry| entry.path())
        .collect();

    if let Some(audio_file) = audio_files.choose(&mut rand::thread_rng()) {
        let file = std::fs::File::open(audio_file)?;
        let source = Decoder::new(std::io::BufReader::new(file))?;

        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        sink.append(source);
        sink.sleep_until_end();
    } else {
        println!("No audio files found for {}", prayer_name);
    }

    Ok(())
}

fn find_upcoming_time<'a>(times: &'a [TimeInfo], current_time: &'a NaiveTime) -> Option<&'a TimeInfo> {
    let current_seconds = current_time.num_seconds_from_midnight();
    let mut next_time: Option<&TimeInfo> = None;
    let mut min_diff = u32::MAX;

    for time_info in times {
        let time_seconds = time_info.time.num_seconds_from_midnight();
        let diff = if time_seconds > current_seconds {
            time_seconds - current_seconds
        } else {
            time_seconds + 24 * 3600 - current_seconds
        };

        if diff < min_diff {
            min_diff = diff;
            next_time = Some(time_info);
        }
    }

    next_time
}

fn calculate_duration(start_time: &NaiveTime, end_time: &NaiveTime) -> Duration {
    let start_seconds = start_time.num_seconds_from_midnight();
    let end_seconds = end_time.num_seconds_from_midnight();

    if end_seconds > start_seconds {
        Duration::seconds((end_seconds - start_seconds) as i64)
    } else {
        Duration::seconds((end_seconds + 24 * 3600 - start_seconds) as i64)
    }
}

fn format_duration(duration: Duration) -> String {
    let hours = duration.num_hours();
    let minutes = (duration.num_minutes() % 60).abs();
    let seconds = (duration.num_seconds() % 60).abs();

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn fetch_prayer_times_from_api() -> Result<PrayerTimesResponse, Box<dyn Error>> {
    let api_url = "https://raspy-lake-0877.adhanapp.workers.dev/";
    
    let response = reqwest::blocking::get(api_url)?;
    
    match response.status().is_success() {
        true => {
            let response_body = response.text()?;
            let prayer_times: PrayerTimesResponse = serde_json::from_str(&response_body)?;
            Ok(prayer_times)
        },
        false => {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to fetch prayer times",
            )))
        }
    }
}
