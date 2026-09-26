use super::provider::WeatherData;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedWeather {
    pub cached_at: DateTime<Utc>,
    #[serde(default)]
    pub latitude: f64,
    #[serde(default)]
    pub longitude: f64,
    pub data: WeatherData,
}

impl CachedWeather {
    pub fn new(data: WeatherData, latitude: f64, longitude: f64) -> Self {
        Self {
            cached_at: Utc::now(),
            latitude,
            longitude,
            data,
        }
    }

    /// Check if coordinates are close enough (within ~2km / 0.02 deg)
    pub fn is_location_match(&self, lat: f64, lon: f64) -> bool {
        (self.latitude - lat).abs() < 0.02 && (self.longitude - lon).abs() < 0.02
    }

    /// Current weather cache valid for 30 minutes (Design Doc Sec 13)
    pub fn is_current_valid(&self) -> bool {
        let age = Utc::now().signed_duration_since(self.cached_at);
        age >= chrono::Duration::zero() && age < chrono::Duration::minutes(30)
    }

    /// Forecast cache valid for 3 hours (Design Doc Sec 13)
    pub fn is_forecast_valid(&self) -> bool {
        let age = Utc::now().signed_duration_since(self.cached_at);
        age >= chrono::Duration::zero() && age < chrono::Duration::hours(3)
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
        let serialized = serde_json::to_string(cached).map_err(std::io::Error::other)?;

        let pid = std::process::id();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("cache");
        let tmp_path = path.with_file_name(format!("{filename}.{pid}.{nanos}.tmp"));

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

        let cached = CachedWeather::new(data.clone(), 35.6895, 139.6917);
        assert!(cached.is_current_valid());
        assert!(cached.is_forecast_valid());
        assert!(cached.is_location_match(35.6895, 139.6917));
        assert!(!cached.is_location_match(51.5074, -0.1278));

        // Test clock skew / future timestamp: age is negative
        let future_cached = CachedWeather {
            cached_at: Utc::now() + chrono::Duration::minutes(10),
            latitude: 35.6895,
            longitude: 139.6917,
            data,
        };
        assert!(!future_cached.is_current_valid());
        assert!(!future_cached.is_forecast_valid());
    }

    #[test]
    fn test_startup_cache_location_and_validity_filtering() {
        let data = WeatherData {
            current: CurrentWeather {
                temperature_celsius: 20.0,
                weather_code: 0,
                condition_text: "Clear sky".into(),
            },
            daily: Vec::new(),
        };

        // Cache created for Tokyo
        let tokyo_cache = CachedWeather::new(data.clone(), 35.6895, 139.6917);

        // When current config is London (51.5074, -0.1278)
        let config_lat = 51.5074;
        let config_lon = -0.1278;

        let filtered = Some(tokyo_cache.clone())
            .filter(|c| c.is_location_match(config_lat, config_lon) && c.is_current_valid());
        assert!(
            filtered.is_none(),
            "Startup weather must NOT display Tokyo cache when configured for London"
        );

        // When current config is Tokyo (matching)
        let filtered_tokyo = Some(tokyo_cache.clone())
            .filter(|c| c.is_location_match(35.6895, 139.6917) && c.is_current_valid());
        assert!(
            filtered_tokyo.is_some(),
            "Must accept matching and fresh Tokyo cache"
        );

        // When cache is expired (>30m old)
        let expired_cache = CachedWeather {
            cached_at: Utc::now() - chrono::Duration::minutes(45),
            latitude: 35.6895,
            longitude: 139.6917,
            data,
        };
        let filtered_expired = Some(expired_cache)
            .filter(|c| c.is_location_match(35.6895, 139.6917) && c.is_current_valid());
        assert!(
            filtered_expired.is_none(),
            "Must reject expired cache on startup"
        );
    }

    #[test]
    fn test_concurrent_cache_writes() {
        let temp_dir =
            std::env::temp_dir().join(format!("toodle_test_conc_{}", std::process::id()));
        let temp_cache = temp_dir.join("weather_cache.json");
        let _ = std::fs::remove_file(&temp_cache);

        let data = WeatherData {
            current: CurrentWeather {
                temperature_celsius: 20.0,
                weather_code: 0,
                condition_text: "Clear sky".into(),
            },
            daily: Vec::new(),
        };

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let p = temp_cache.clone();
                let d = data.clone();
                std::thread::spawn(move || {
                    let cached = CachedWeather::new(d, 35.0 + i as f64 * 0.1, 139.0);
                    WeatherCache::save_to(&cached, &p)
                })
            })
            .collect();

        for h in handles {
            let res = h.join().expect("thread should not panic");
            assert!(res.is_ok(), "Concurrent save must succeed: {:?}", res);
        }

        // Cache file should exist and be valid JSON
        let loaded = WeatherCache::load_from(&temp_cache);
        assert!(
            loaded.is_some(),
            "Cache must be loadable and non-corrupted after concurrent writes"
        );

        let _ = std::fs::remove_file(&temp_cache);
        let _ = std::fs::remove_dir(&temp_dir);
    }
}
