struct OpenMeteo {
    latitude: f64,
    longitude: f64,
    generationtime_ms: f64,
    utc_offset_second: u32,
    timezone: String,
    timezone_abbreviation: String,
    elevation: f64,
    current_units: CurrentUnits,
    current: Current,
}

struct CurrentUnits {
    time: String,
    interval: String,
    temperature_2m: String,
    relative_humidity_2m: String,
    apparent_temperature: String,
    precipitation: String,
    rain: String,
    showers: String,
    snowfall: String,
    weather_code: String,
    cloud_cover: String,
    wind_speed_10m: String,
    wind_direction_10m: String,
    wind_gusts_10m: String,
}

struct Current {
    time: u64,
    interval: u32,
    temperature_2m: f64,
    relative_humidity_2m: i32,
    apparent_temperature: f64,
    precipitation: f64,
    rain: f64,
    showers: f64,
    snowfall: f64,
    weather_code: u8,
    cloud_cover: u16,
}

fn main() {
    println!("Hello, world!");
}
