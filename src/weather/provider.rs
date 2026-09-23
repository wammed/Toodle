use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CurrentWeather {
    pub temperature_celsius: f32,
    pub weather_code: u8,
    pub condition_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DailyForecast {
    pub date: String,
    pub weather_code: u8,
    pub condition_text: String,
    pub temp_min_celsius: f32,
    pub temp_max_celsius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeatherData {
    pub current: CurrentWeather,
    pub daily: Vec<DailyForecast>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WeatherError {
    Network(String),
    RateLimited(String),
    Parse(String),
    Unavailable,
}

impl std::fmt::Display for WeatherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeatherError::Network(s) => write!(f, "Network error: {}", s),
            WeatherError::RateLimited(s) => write!(f, "Rate limited (429): {}", s),
            WeatherError::Parse(s) => write!(f, "Parse error: {}", s),
            WeatherError::Unavailable => write!(f, "Weather unavailable"),
        }
    }
}

impl std::error::Error for WeatherError {}

#[async_trait]
pub trait WeatherProvider: Send + Sync {
    async fn fetch_weather(&self, lat: f64, lon: f64) -> Result<WeatherData, WeatherError>;
}

pub fn wmo_code_to_text(code: u8) -> &'static str {
    match code {
        0 => "Clear sky",
        1 => "Mainly clear",
        2 => "Partly cloudy",
        3 => "Overcast",
        45 | 48 => "Fog",
        51 | 53 | 55 => "Drizzle",
        56 | 57 => "Freezing Drizzle",
        61 | 63 | 65 => "Rain",
        66 | 67 => "Freezing Rain",
        71 | 73 | 75 | 77 => "Snow",
        80..=82 => "Rain showers",
        85 | 86 => "Snow showers",
        95 => "Thunderstorm",
        96 | 99 => "Thunderstorm w/ hail",
        _ => "Cloudy",
    }
}

pub fn wmo_code_to_icon(code: u8) -> crate::weather::icon::WeatherIcon {
    crate::weather::icon::WeatherIcon::from_wmo_code(code)
}

pub struct OpenMeteoProvider {
    client: reqwest::Client,
}

impl OpenMeteoProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }
}

impl Default for OpenMeteoProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
pub(crate) struct OpenMeteoResponse {
    pub current: Option<OpenMeteoCurrent>,
    pub daily: Option<OpenMeteoDaily>,
}

#[derive(Deserialize)]
pub(crate) struct OpenMeteoCurrent {
    pub temperature_2m: Option<f32>,
    pub weather_code: Option<u8>,
}

#[derive(Deserialize)]
pub(crate) struct OpenMeteoDaily {
    pub time: Option<Vec<String>>,
    pub weather_code: Option<Vec<u8>>,
    pub temperature_2m_max: Option<Vec<f32>>,
    pub temperature_2m_min: Option<Vec<f32>>,
}

pub(crate) fn parse_open_meteo_response(
    om: OpenMeteoResponse,
) -> Result<WeatherData, WeatherError> {
    let current_data = om
        .current
        .ok_or_else(|| WeatherError::Parse("Missing 'current' in Open-Meteo response".into()))?;

    let temp = current_data.temperature_2m.ok_or_else(|| {
        WeatherError::Parse("Missing 'current.temperature_2m' in Open-Meteo response".into())
    })?;
    let code = current_data.weather_code.ok_or_else(|| {
        WeatherError::Parse("Missing 'current.weather_code' in Open-Meteo response".into())
    })?;
    let condition_text = wmo_code_to_text(code).to_string();

    let mut daily_forecasts = Vec::new();
    if let Some(daily) = om.daily {
        let times = daily.time.unwrap_or_default();
        let codes = daily.weather_code.unwrap_or_default();
        let maxs = daily.temperature_2m_max.unwrap_or_default();
        let mins = daily.temperature_2m_min.unwrap_or_default();

        let len = times.len();
        if codes.len() != len || maxs.len() != len || mins.len() != len {
            return Err(WeatherError::Parse(format!(
                "Mismatched daily forecast array lengths: time={}, codes={}, maxs={}, mins={}",
                len,
                codes.len(),
                maxs.len(),
                mins.len()
            )));
        }

        for i in 0..len {
            let d_code = codes[i];
            daily_forecasts.push(DailyForecast {
                date: times[i].clone(),
                weather_code: d_code,
                condition_text: wmo_code_to_text(d_code).to_string(),
                temp_min_celsius: mins[i],
                temp_max_celsius: maxs[i],
            });
        }
    }

    Ok(WeatherData {
        current: CurrentWeather {
            temperature_celsius: temp,
            weather_code: code,
            condition_text,
        },
        daily: daily_forecasts,
    })
}

pub fn parse_open_meteo_json(json_str: &str) -> Result<WeatherData, WeatherError> {
    let om: OpenMeteoResponse =
        serde_json::from_str(json_str).map_err(|e| WeatherError::Parse(e.to_string()))?;
    parse_open_meteo_response(om)
}

#[async_trait]
impl WeatherProvider for OpenMeteoProvider {
    async fn fetch_weather(&self, lat: f64, lon: f64) -> Result<WeatherData, WeatherError> {
        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weather_code&daily=weather_code,temperature_2m_max,temperature_2m_min&timezone=auto",
            lat, lon
        );

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| WeatherError::Network(e.to_string()))?;

        if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(WeatherError::RateLimited("429 Too Many Requests".into()));
        }

        if !resp.status().is_success() {
            return Err(WeatherError::Network(format!(
                "HTTP status: {}",
                resp.status()
            )));
        }

        let om: OpenMeteoResponse = resp
            .json()
            .await
            .map_err(|e| WeatherError::Parse(e.to_string()))?;

        parse_open_meteo_response(om)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wmo_code_to_text() {
        assert_eq!(wmo_code_to_text(0), "Clear sky");
        assert_eq!(wmo_code_to_text(1), "Mainly clear");
        assert_eq!(wmo_code_to_text(2), "Partly cloudy");
        assert_eq!(wmo_code_to_text(3), "Overcast");
        assert_eq!(wmo_code_to_text(56), "Freezing Drizzle");
        assert_eq!(wmo_code_to_text(57), "Freezing Drizzle");
        assert_eq!(wmo_code_to_text(61), "Rain");
        assert_eq!(wmo_code_to_text(71), "Snow");
        assert_eq!(wmo_code_to_text(95), "Thunderstorm");
        assert_eq!(wmo_code_to_text(255), "Cloudy");
    }

    #[test]
    fn test_wmo_code_to_icon() {
        use crate::weather::icon::WeatherIcon;
        assert_eq!(wmo_code_to_icon(0), WeatherIcon::Clear);
        assert_eq!(wmo_code_to_icon(1), WeatherIcon::PartlyCloudy);
        assert_eq!(wmo_code_to_icon(2), WeatherIcon::PartlyCloudy);
        assert_eq!(wmo_code_to_icon(3), WeatherIcon::Cloudy);
        assert_eq!(wmo_code_to_icon(45), WeatherIcon::Fog);
        assert_eq!(wmo_code_to_icon(53), WeatherIcon::Drizzle);
        assert_eq!(wmo_code_to_icon(56), WeatherIcon::FreezingDrizzle);
        assert_eq!(wmo_code_to_icon(57), WeatherIcon::FreezingDrizzle);
        assert_eq!(wmo_code_to_icon(63), WeatherIcon::Rain);
        assert_eq!(wmo_code_to_icon(66), WeatherIcon::FreezingRain);
        assert_eq!(wmo_code_to_icon(71), WeatherIcon::Snow);
        assert_eq!(wmo_code_to_icon(80), WeatherIcon::RainShower);
        assert_eq!(wmo_code_to_icon(85), WeatherIcon::SnowShower);
        assert_eq!(wmo_code_to_icon(95), WeatherIcon::Thunderstorm);
        assert_eq!(wmo_code_to_icon(255), WeatherIcon::Cloudy);
    }

    #[test]
    fn test_parse_open_meteo_json_valid() {
        let json = r#"{
            "current": {
                "temperature_2m": 21.5,
                "weather_code": 1
            },
            "daily": {
                "time": ["2026-09-24", "2026-09-25"],
                "weather_code": [0, 61],
                "temperature_2m_max": [25.0, 22.0],
                "temperature_2m_min": [18.0, 16.0]
            }
        }"#;

        let res = parse_open_meteo_json(json).expect("Should parse valid response");
        assert_eq!(res.current.temperature_celsius, 21.5);
        assert_eq!(res.current.weather_code, 1);
        assert_eq!(res.current.condition_text, "Mainly clear");
        assert_eq!(res.daily.len(), 2);
        assert_eq!(res.daily[0].date, "2026-09-24");
        assert_eq!(res.daily[0].temp_max_celsius, 25.0);
        assert_eq!(res.daily[0].temp_min_celsius, 18.0);
        assert_eq!(res.daily[0].condition_text, "Clear sky");
        assert_eq!(res.daily[1].condition_text, "Rain");
    }

    #[test]
    fn test_parse_open_meteo_json_missing_temperature() {
        let json = r#"{
            "current": {
                "weather_code": 0
            }
        }"#;

        let err = parse_open_meteo_json(json).unwrap_err();
        match err {
            WeatherError::Parse(msg) => {
                assert!(msg.contains("temperature_2m"), "Error must mention temperature_2m: {msg}");
            }
            _ => panic!("Expected WeatherError::Parse, got {:?}", err),
        }
    }

    #[test]
    fn test_parse_open_meteo_json_missing_weather_code() {
        let json = r#"{
            "current": {
                "temperature_2m": 15.0
            }
        }"#;

        let err = parse_open_meteo_json(json).unwrap_err();
        match err {
            WeatherError::Parse(msg) => {
                assert!(msg.contains("weather_code"), "Error must mention weather_code: {msg}");
            }
            _ => panic!("Expected WeatherError::Parse, got {:?}", err),
        }
    }

    #[test]
    fn test_parse_open_meteo_json_mismatched_daily_arrays() {
        let json = r#"{
            "current": {
                "temperature_2m": 19.0,
                "weather_code": 2
            },
            "daily": {
                "time": ["2026-09-24", "2026-09-25"],
                "weather_code": [0],
                "temperature_2m_max": [25.0, 22.0],
                "temperature_2m_min": [18.0, 16.0]
            }
        }"#;

        let err = parse_open_meteo_json(json).unwrap_err();
        match err {
            WeatherError::Parse(msg) => {
                assert!(msg.contains("Mismatched daily forecast array lengths"), "Expected length mismatch error: {msg}");
            }
            _ => panic!("Expected WeatherError::Parse, got {:?}", err),
        }
    }
}
