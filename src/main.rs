use chrono::{NaiveTime, Timelike, Duration, Datelike};
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
use toml;

#[derive(Debug, Deserialize)]
struct Config {
    api_url: String,
    mpd: MpdConfig,
}

#[derive(Debug, Deserialize)]
struct MpdConfig {
    volume: u8,
}

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

#[derive(Debug, Deserialize, Serialize)]
struct YearPrayerTimes {
    city: Option<String>,
    times: std::collections::HashMap<String, DayTimes>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DayTimes {
    date: Option<String>,
    fajr: Option<String>,
    dhuhr: Option<String>,
    asr: Option<String>,
    magrib: Option<String>,
    isha: Option<String>,
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

    // Spawn a background thread that wakes at 1:00 AM every day and, on the last
    // day of the year (31 December), downloads the following year's prayer times
    // file. This keeps internet use to a single annual fetch while ensuring the
    // app has data ready before the new year begins.
    let _ = thread::spawn(|| {
        loop {
            let current_time = chrono::Local::now().time();
            let wake_time = match NaiveTime::from_hms_opt(1, 0, 0) {
                Some(t) => t,
                None => {
                    eprintln!("Invalid wake time specified.");
                    return;
                }
            };

            let duration_until_wake = calculate_duration(&current_time, &wake_time);
            println!(
                "Background renewal thread sleeping for: {}",
                format_duration(duration_until_wake)
            );
            thread::sleep(duration_until_wake.to_std().unwrap());

            // Only act on 31 December — fetch the next year's file.
            let now = chrono::Local::now();
            if now.month() == 12 && now.day() == 31 {
                println!("Last day of year — fetching next year's prayer times.");
                if let Err(err) = fetch_and_process_prayer_times() {
                    eprintln!("Error fetching next year's prayer times: {}", err);
                }
            } else {
                println!("Background renewal check: not year-end, nothing to do.");
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

                        let config = read_config().unwrap();
                        set_rhythmbox_volume(0); // Turn volume off
                        set_mpd_volume(0); // Set MPD volume to 0
                    
                        play_adhan(&time_info.info).unwrap();

                        set_rhythmbox_volume(1); // Turn volume on
                        set_mpd_volume(config.mpd.volume); // Restore MPD volume from config
                        
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
    // When called on 31 December we want to pre-fetch the *next* year's file so
    // the app is ready to go on 1 January without needing internet access.
    let now = chrono::Local::now();
    let target_year = if now.month() == 12 && now.day() == 31 {
        now.year() + 1
    } else {
        now.year()
    };

    let home_dir = home_dir().ok_or(std::io::Error::new(std::io::ErrorKind::Other, "Failed to determine home directory"))?;
    let file_path = home_dir.join(format!("adhanapp/prayer_times_{}.json", target_year));

    if file_path.exists() {
        // Already have the target year's file — nothing to do.
        println!("Prayer times file for {} already exists.", target_year);
        return Ok(());
    }

    println!("Fetching prayer times for {} from API.", target_year);

    // Try to fetch the yearly file from the API. If this fails but an older
    // prayer_times file exists, keep using it (resilience to internet outages).
    match fetch_year_prayer_times_from_api() {
        Ok(year_data) => {
            save_year_prayer_times_to_file(&year_data, target_year)?;
            println!("Prayer times for {} saved successfully.", target_year);
            Ok(())
        }
        Err(e) => {
            // Look for any existing prayer_times_*.json file and keep using it.
            let dir = home_dir.join("adhanapp");
            if dir.exists() {
                if let Ok(entries) = fs::read_dir(&dir) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let p = entry.path();
                        if p.is_file() {
                            if let Some(fname) = p.file_name().and_then(|s| s.to_str()) {
                                if fname.starts_with("prayer_times_") && fname.ends_with(".json") {
                                    println!("Fetch failed but found fallback file: {}", fname);
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
            }

            // No fallback file found — surface the original error.
            Err(e)
        }
    }
}

fn save_year_prayer_times_to_file(year_data: &YearPrayerTimes, year: i32) -> Result<(), Box<dyn Error>> {
    let prayer_times_json = serde_json::to_string(year_data)?;
    let home_dir = home_dir().ok_or(std::io::Error::new(std::io::ErrorKind::Other, "Failed to determine home directory"))?;
    let file_path = home_dir.join(format!("adhanapp/prayer_times_{}.json", year));
    fs::write(&file_path, prayer_times_json)?;
    Ok(())
}

fn fetch_prayer_times_from_file() -> Result<PrayerTimesResponse, Box<dyn Error>> {
    let home_dir = home_dir().ok_or(std::io::Error::new(std::io::ErrorKind::Other, "Failed to determine home directory"))?;
    let year = chrono::Local::now().year();
    let file_path = home_dir.join(format!("adhanapp/prayer_times_{}.json", year));

    // Prefer current year file
    if file_path.exists() {
        let prayer_times_json = fs::read_to_string(&file_path)?;
        let year_data: YearPrayerTimes = serde_json::from_str(&prayer_times_json)?;
        let today = chrono::Local::now().date_naive().to_string();
        if let Some(day) = year_data.times.get(&today) {
            let res = PrayerTimesResponse {
                fajr: day.fajr.clone().unwrap_or_default(),
                dhuhr: day.dhuhr.clone().unwrap_or_default(),
                asr: day.asr.clone().unwrap_or_default(),
                magrib: day.magrib.clone().unwrap_or_default(),
                isha: day.isha.clone().unwrap_or_default(),
            };
            return Ok(res);
        } else {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Today's times not found in year file")));
        }
    }

    // Fallback: look for any existing prayer_times_*.json file
    let dir = home_dir.join("adhanapp");
    if dir.exists() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.is_file() {
                    if let Some(fname) = p.file_name().and_then(|s| s.to_str()) {
                        if fname.starts_with("prayer_times_") && fname.ends_with(".json") {
                            let prayer_times_json = fs::read_to_string(&p)?;
                            if let Ok(year_data) = serde_json::from_str::<YearPrayerTimes>(&prayer_times_json) {
                                let today = chrono::Local::now().date_naive().to_string();
                                if let Some(day) = year_data.times.get(&today) {
                                    let res = PrayerTimesResponse {
                                        fajr: day.fajr.clone().unwrap_or_default(),
                                        dhuhr: day.dhuhr.clone().unwrap_or_default(),
                                        asr: day.asr.clone().unwrap_or_default(),
                                        magrib: day.magrib.clone().unwrap_or_default(),
                                        isha: day.isha.clone().unwrap_or_default(),
                                    };
                                    return Ok(res);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "No prayer times file found")))
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

        // Only consider prayer times that are strictly in the future today.
        // Once all prayers have passed, return None so the outer loop re-fetches
        // the next day's times from the year file instead of wrapping around to
        // yesterday's Fajr.
        if time_seconds <= current_seconds {
            continue;
        }

        let diff = time_seconds - current_seconds;
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

fn read_config() -> Result<Config, Box<dyn Error>> {
    let home_dir = home_dir().ok_or("Failed to determine home directory")?;
    let config_path = home_dir.join("adhanapp/config.toml");
    let config_str = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&config_str)?;
    Ok(config)
}

fn fetch_year_prayer_times_from_api() -> Result<YearPrayerTimes, Box<dyn Error>> {
    let config = read_config()?;
    let api_url = &config.api_url;

    let year_url = if api_url.ends_with('/') {
        format!("{}year", api_url)
    } else {
        format!("{}/year", api_url)
    };

    let response = reqwest::blocking::get(&year_url)?;
    if response.status().is_success() {
        let response_body = response.text()?;
        let year_data: YearPrayerTimes = serde_json::from_str(&response_body)?;
        Ok(year_data)
    } else {
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Failed to fetch yearly prayer times")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;

    fn t(h: u32, m: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(h, m, 0).unwrap()
    }

    fn sample_times() -> Vec<TimeInfo> {
        vec![
            TimeInfo { time: t(5, 30),  info: "Fajr".into() },
            TimeInfo { time: t(13, 0),  info: "Dhuhr".into() },
            TimeInfo { time: t(16, 30), info: "Asr".into() },
            TimeInfo { time: t(19, 45), info: "Magrib".into() },
            TimeInfo { time: t(21, 15), info: "Isha".into() },
        ]
    }

    #[test]
    fn picks_next_prayer_during_day() {
        let now = t(14, 0); // between Dhuhr and Asr
        let times = sample_times();
        let next = find_upcoming_time(&times, &now).unwrap();
        assert_eq!(next.info, "Asr");
    }

    #[test]
    fn picks_first_prayer_of_day() {
        let now = t(3, 0); // before Fajr
        let times = sample_times();
        let next = find_upcoming_time(&times, &now).unwrap();
        assert_eq!(next.info, "Fajr");
    }

    #[test]
    fn returns_none_after_isha() {
        // Regression test for the Fajr wrap-around bug — old code would return
        // today's Fajr here instead of None, causing yesterday's time to fire.
        let now = t(22, 0); // after all prayers
        let times = sample_times();
        let result = find_upcoming_time(&times, &now);
        assert!(result.is_none(), "Expected None after last prayer, got {:?}", result.map(|t| &t.info));
    }

    #[test]
    fn picks_isha_just_before_it() {
        let now = t(21, 14); // 1 minute before Isha
        let times = sample_times();
        let next = find_upcoming_time(&times, &now).unwrap();
        assert_eq!(next.info, "Isha");
    }

    #[test]
    fn returns_none_exactly_at_isha() {
        // The prayer fires at the exact second, so that moment is no longer upcoming.
        let now = t(21, 15);
        let times = sample_times();
        let result = find_upcoming_time(&times, &now);
        assert!(result.is_none());
    }
}