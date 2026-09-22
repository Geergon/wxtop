mod openmeteo;

use color_eyre::eyre::{bail, eyre};
use openmeteo::*;
use serde::{Deserialize, Serialize};

use crate::{
    Model,
    config::{self, Config},
};

pub enum WeatherApi {
    OpenMeteo,
}

pub enum WeatherApiResponse {
    OpenMeteo(OpenMeteo),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeoLocation {
    #[serde(rename = "type")]
    pub collection_type: String,
    pub features: Vec<Feature>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Feature {
    #[serde(rename = "type")]
    pub feature_type: String,
    pub properties: Properties,
    pub geometry: Geometry,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Properties {
    pub osm_type: String,
    pub osm_id: u64,
    pub osm_key: String,
    pub osm_value: String,
    pub name: String,
    pub country: String,
    pub countrycode: String,

    pub r#type: Option<String>,
    pub city: Option<String>,
    pub district: Option<String>,
    pub street: Option<String>,
    pub housenumber: Option<String>,
    pub postcode: Option<String>,
    pub locality: Option<String>,
    pub county: Option<String>,
    pub state: Option<String>,
    pub extra: Option<Extra>,
    pub extent: Option<Vec<f64>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Extra {
    pub admin_level: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Geometry {
    #[serde(rename = "type")]
    pub geometry_type: String,
    pub coordinates: (f64, f64),
}

pub async fn fetch_forecast(
    provider: WeatherApi,
    url: String,
) -> color_eyre::Result<WeatherApiResponse> {
    let res = reqwest::get(url).await?.text().await?;

    match provider {
        WeatherApi::OpenMeteo => {
            let openmeteo: OpenMeteo = serde_json::from_str(&res)?;
            return Ok(WeatherApiResponse::OpenMeteo(openmeteo));
        }
    }
}

pub async fn fetch_location(query: &str) -> color_eyre::Result<GeoLocation> {
    let url = format!(
        "https://photon.komoot.io/api/?q={query}&limit=5&layer=city&layer=locality&lang=en"
    );
    let Ok(res) = reqwest::get(url).await else {
        bail!("failed to get response from geocoding API")
    };
    let Ok(content) = res.text().await else {
        bail!("failed to decode response into text")
    };

    let locations: GeoLocation = serde_json::from_str(&content)?;
    Ok(locations)
}
