use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OpenMeteo {
    pub latitude: f64,
    pub longitude: f64,
    pub generationtime_ms: f64,
    pub utc_offset_seconds: i32,
    pub timezone: String,
    pub timezone_abbreviation: String,
    pub elevation: f64,
    pub current_units: CurrentUnits,
    pub current: Current,
    pub hourly_units: HourlyUnits,
    pub hourly: Hourly,
    pub daily_units: DailyUnits,
    pub daily: Daily,
}

#[derive(Debug, Deserialize)]
pub struct CurrentUnits {
    pub time: String,
    pub interval: String,
    pub temperature_2m: String,
    pub relative_humidity_2m: String,
    pub apparent_temperature: String,
    pub precipitation: String,
    pub rain: String,
    pub showers: String,
    pub snowfall: String,
    pub weather_code: String,
    pub cloud_cover: String,
    pub wind_speed_10m: String,
    pub wind_direction_10m: String,
    pub wind_gusts_10m: String,
}

#[derive(Debug, Deserialize)]
pub struct Current {
    pub time: i64,
    pub interval: u32,
    pub temperature_2m: f64,
    pub relative_humidity_2m: i32,
    pub apparent_temperature: f64,
    pub precipitation: f64,
    pub rain: f64,
    pub showers: f64,
    pub snowfall: f64,
    pub weather_code: u8,
    pub cloud_cover: u16,
    pub wind_speed_10m: f64,
    pub wind_direction_10m: u16,
    pub wind_gusts_10m: f64,
}

#[derive(Debug, Deserialize)]
pub struct HourlyUnits {
    pub time: String,
    pub temperature_2m: String,
    pub relative_humidity_2m: String,
    pub apparent_temperature: String,
    pub precipitation_probability: String,
    pub precipitation: String,
    pub weather_code: String,
    pub cloud_cover: String,
    pub visibility: String,
    pub wind_speed_10m: String,
    pub wind_direction_10m: String,
}

#[derive(Debug, Deserialize)]
pub struct Hourly {
    pub time: Vec<i64>,
    pub temperature_2m: Vec<f64>,
    pub relative_humidity_2m: Vec<u16>,
    pub apparent_temperature: Vec<f64>,
    pub precipitation_probability: Vec<u16>,
    pub precipitation: Vec<f64>,
    pub weather_code: Vec<u8>,
    pub cloud_cover: Vec<u16>,
    pub visibility: Vec<f64>,
    pub wind_speed_10m: Vec<f64>,
    pub wind_direction_10m: Vec<u16>,
}

#[derive(Debug, Deserialize)]
pub struct DailyUnits {
    pub time: String,
    pub weather_code: String,
    pub temperature_2m_max: String,
    pub temperature_2m_min: String,
    pub precipitation_sum: String,
    pub rain_sum: String,
    pub showers_sum: String,
    pub snowfall_sum: String,
    pub precipitation_hours: String,
    pub precipitation_probability_max: String,
    pub wind_speed_10m_max: String,
    pub wind_gusts_10m_max: String,
    pub wind_direction_10m_dominant: String,
}

#[derive(Debug, Deserialize)]
pub struct Daily {
    pub time: Vec<i64>,
    pub weather_code: Vec<u8>,
    pub temperature_2m_max: Vec<f64>,
    pub temperature_2m_min: Vec<f64>,
    pub precipitation_sum: Vec<f64>,
    pub rain_sum: Vec<f64>,
    pub showers_sum: Vec<f64>,
    pub snowfall_sum: Vec<f64>,
    pub precipitation_hours: Vec<f64>,
    pub precipitation_probability_max: Vec<u16>,
    pub wind_speed_10m_max: Vec<f64>,
    pub wind_gusts_10m_max: Vec<f64>,
    pub wind_direction_10m_dominant: Vec<u16>,
}

fn main() {
    println!("Hello, world!");
}
