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
        61 | 63 | 65 => "Rain",
        66 | 67 => "Freezing Rain",
        71 | 73 | 75 | 77 => "Snow",
        80 | 81 | 82 => "Rain showers",
        85 | 86 => "Snow showers",
        95 => "Thunderstorm",
        96 | 99 => "Thunderstorm w/ hail",
        _ => "Cloudy",
    }
}

pub fn wmo_code_to_emoji(code: u8) -> &'static str {
    match code {
        0 => "☀️",
        1 => "🌤️",
        2 => "⛅",
        3 => "☁️",
        45 | 48 => "🌫️",
        51 | 53 | 55 => "🌦️",
        61 | 63 | 65 => "🌧️",
        66 | 67 => "🌧️❄️",
        71 | 73 | 75 | 77 => "❄️",
        80 | 81 | 82 => "🌧️",
        85 | 86 => "🌨️",
        95 => "⛈️",
        96 | 99 => "⛈️",
        _ => "☁️",
    }
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
struct OpenMeteoResponse {
    current: Option<OpenMeteoCurrent>,
    daily: Option<OpenMeteoDaily>,
}

#[derive(Deserialize)]
struct OpenMeteoCurrent {
    temperature_2m: Option<f32>,
    weather_code: Option<u8>,
}

#[derive(Deserialize)]
struct OpenMeteoDaily {
    time: Option<Vec<String>>,
    weather_code: Option<Vec<u8>>,
    temperature_2m_max: Option<Vec<f32>>,
    temperature_2m_min: Option<Vec<f32>>,
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

        let current_data = om.current.ok_or_else(|| {
            WeatherError::Parse("Missing 'current' in Open-Meteo response".into())
        })?;

        let temp = current_data.temperature_2m.unwrap_or(0.0);
        let code = current_data.weather_code.unwrap_or(0);
        let condition_text = wmo_code_to_text(code).to_string();

        let mut daily_forecasts = Vec::new();
        if let Some(daily) = om.daily {
            let times = daily.time.unwrap_or_default();
            let codes = daily.weather_code.unwrap_or_default();
            let maxs = daily.temperature_2m_max.unwrap_or_default();
            let mins = daily.temperature_2m_min.unwrap_or_default();

            for i in 0..times.len() {
                let d_code = codes.get(i).copied().unwrap_or(0);
                daily_forecasts.push(DailyForecast {
                    date: times.get(i).cloned().unwrap_or_default(),
                    weather_code: d_code,
                    condition_text: wmo_code_to_text(d_code).to_string(),
                    temp_min_celsius: mins.get(i).copied().unwrap_or(0.0),
                    temp_max_celsius: maxs.get(i).copied().unwrap_or(0.0),
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
        assert_eq!(wmo_code_to_text(61), "Rain");
        assert_eq!(wmo_code_to_text(71), "Snow");
        assert_eq!(wmo_code_to_text(95), "Thunderstorm");
        assert_eq!(wmo_code_to_text(255), "Cloudy");
    }

    #[test]
    fn test_wmo_code_to_emoji() {
        assert_eq!(wmo_code_to_emoji(0), "☀️");
        assert_eq!(wmo_code_to_emoji(1), "🌤️");
        assert_eq!(wmo_code_to_emoji(2), "⛅");
        assert_eq!(wmo_code_to_emoji(3), "☁️");
        assert_eq!(wmo_code_to_emoji(45), "🌫️");
        assert_eq!(wmo_code_to_emoji(53), "🌦️");
        assert_eq!(wmo_code_to_emoji(63), "🌧️");
        assert_eq!(wmo_code_to_emoji(71), "❄️");
        assert_eq!(wmo_code_to_emoji(80), "🌧️");
        assert_eq!(wmo_code_to_emoji(85), "🌨️");
        assert_eq!(wmo_code_to_emoji(95), "⛈️");
        assert_eq!(wmo_code_to_emoji(255), "☁️");
    }
}
