use super::provider::WeatherData;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedWeather {
    pub cached_at: DateTime<Utc>,
    pub data: WeatherData,
}

impl CachedWeather {
    pub fn new(data: WeatherData) -> Self {
        Self {
            cached_at: Utc::now(),
            data,
        }
    }

    /// Current weather cache valid for 30 minutes (Design Doc Sec 13)
    pub fn is_current_valid(&self) -> bool {
        let age = Utc::now().signed_duration_since(self.cached_at);
        age < chrono::Duration::minutes(30)
    }

    /// Forecast cache valid for 3 hours (Design Doc Sec 13)
    pub fn is_forecast_valid(&self) -> bool {
        let age = Utc::now().signed_duration_since(self.cached_at);
        age < chrono::Duration::hours(3)
    }
}

pub struct WeatherCache;

impl WeatherCache {
    pub fn cache_file_path() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join("toodle")
            .join("weather_cache.json")
    }

    pub fn load() -> Option<CachedWeather> {
        Self::load_from(&Self::cache_file_path())
    }

    pub fn load_from(path: &std::path::Path) -> Option<CachedWeather> {
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(content) => match serde_json::from_str::<CachedWeather>(&content) {
                    Ok(cached) => {
                        info!("Loaded weather cache from {:?}", path);
                        return Some(cached);
                    }
                    Err(e) => warn!("Failed to parse weather cache at {:?}: {}", path, e),
                },
                Err(e) => warn!("Failed to read weather cache at {:?}: {}", path, e),
            }
        }
        None
    }

    pub fn save(cached: &CachedWeather) -> Result<(), std::io::Error> {
        Self::save_to(cached, &Self::cache_file_path())
    }

    pub fn save_to(cached: &CachedWeather, path: &std::path::Path) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let serialized = serde_json::to_string(cached)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let tmp_path = path.with_extension("tmp");
        fs::write(&tmp_path, serialized)?;
        fs::rename(&tmp_path, path)?;
        info!("Saved weather cache to {:?}", path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::weather::provider::CurrentWeather;

    #[test]
    fn test_cache_validity() {
        let data = WeatherData {
            current: CurrentWeather {
                temperature_celsius: 20.0,
                weather_code: 0,
                condition_text: "Clear sky".into(),
            },
            daily: Vec::new(),
        };

        let cached = CachedWeather::new(data);
        assert!(cached.is_current_valid());
        assert!(cached.is_forecast_valid());
    }
}
