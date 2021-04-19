use std::convert::TryFrom;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SECONDS_IN_DAY: u64 = 86400;
const CONFIG_FILENAME: &str = ".countdown.yml";

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct CountdownConfig {
  events: Vec<Event>,
}

impl Default for CountdownConfig {
  fn default() -> Self {
    Self { events: Vec::new() }
  }
}


#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Event {
  name: String,
  // Unix timestamp (seconds)
  time: u32,
}

impl Event {
  fn days_left(&self, current_time: SystemTime) -> Result<u16, String> {
    let future_time = UNIX_EPOCH + Duration::from_secs(self.time.into());

    match future_time.duration_since(current_time) {
      Ok(dur) => u16::try_from(dur.as_secs() / SECONDS_IN_DAY)
        .map_err(|e| format!("Error calculating days between: {:?}", e)),
      Err(e) => Err(format!("{:?}", e)),
    }
  }
}

fn main() {
  let now = SystemTime::now();
  let result: Result<Vec<(i32, String)>, String> =
    dirs::home_dir()
      .ok_or_else(|| "Failed to find home directory.".to_string())
      .map(|home| format!("{:?}/{:?}", home.to_str(), CONFIG_FILENAME))
      .and_then(|config_file: String| {
        println!("config file: {:?}", config_file);
        confy::load_path(config_file.as_str())
          .map_err(|_| "Couldn't load config.".to_string())})
      .map(|config: CountdownConfig| {
        println!("config: {:?}", config);
        config.events
        .iter()
        .map(|ev| match ev.days_left(now) {
          Ok(days) => (i32::from(days), format!("{:?} days until {:?}", days, ev.name)),
          Err(e) => (-1, (format!("countdown: {:?}", e))),
        }).collect()});

  println!("result: {:?}", result);
  match result  {
      Ok(all_events) => {
        let (mut valid, invalid): (Vec<(i32, String)>, Vec<(i32, String)>) =
          all_events.into_iter().partition(|(days, _)| days > &-1);

        invalid.iter().for_each(|(_, msg)| eprintln!("{:?}", msg));
        valid
          .sort_by(|(a_days, _), (b_days, _)| a_days.cmp(b_days));
        valid.iter().for_each(|(_, msg)| println!("{:?}", msg))
      },
      Err(e) => eprintln!("{:?}", e),
    }
}
