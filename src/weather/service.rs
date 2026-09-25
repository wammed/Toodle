use super::cache::{CachedWeather, WeatherCache};
use super::provider::{OpenMeteoProvider, WeatherData, WeatherError, WeatherProvider};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, warn};

pub struct WeatherService {
    provider: Arc<dyn WeatherProvider>,
    cache_path: Option<PathBuf>,
}

impl WeatherService {
    pub fn new() -> Self {
        Self {
            provider: Arc::new(OpenMeteoProvider::new()),
            cache_path: None,
        }
    }

    pub fn with_provider(provider: Arc<dyn WeatherProvider>) -> Self {
        Self {
            provider,
            cache_path: None,
        }
    }

    pub fn with_provider_and_cache(
        provider: Arc<dyn WeatherProvider>,
        cache_path: PathBuf,
    ) -> Self {
        Self {
            provider,
            cache_path: Some(cache_path),
        }
    }

    fn cache_path(&self) -> PathBuf {
        self.cache_path
            .clone()
            .unwrap_or_else(WeatherCache::cache_file_path)
    }

    /// Fetch weather honoring cache and offline fallback (Design Doc Sec 13 & 14)
    pub async fn get_weather(&self, lat: f64, lon: f64) -> Result<WeatherData, WeatherError> {
        if !(-90.0..=90.0).contains(&lat)
            || !(-180.0..=180.0).contains(&lon)
            || lat.is_nan()
            || lon.is_nan()
        {
            return Err(WeatherError::Parse(format!(
                "Invalid coordinates: lat={}, lon={}. Latitude must be [-90, 90] and longitude [-180, 180].",
                lat, lon
            )));
        }

        let cache_p = self.cache_path();

        // 1. Check local persistent cache
        if let Some(cached) = WeatherCache::load_from(&cache_p)
            .filter(|c| c.is_location_match(lat, lon) && c.is_current_valid())
        {
            info!(
                "Using fresh weather cache (< 30 minutes old) for ({}, {})",
                lat, lon
            );
            return Ok(cached.data);
        }

        // 2. Fetch fresh weather from provider
        match self.provider.fetch_weather(lat, lon).await {
            Ok(data) => {
                let cached = CachedWeather::new(data.clone(), lat, lon);
                if let Err(e) = WeatherCache::save_to(&cached, &cache_p) {
                    warn!("Failed to persist weather cache to {:?}: {}", cache_p, e);
                }
                Ok(data)
            }
            Err(err) => {
                warn!(
                    "Weather fetch failed: {}. Checking for stale cache fallback...",
                    err
                );
                // 3. Fallback to last available cache if available for this location (Sec 13)
                if let Some(stale) = WeatherCache::load_from(&cache_p)
                    .filter(|s| s.is_location_match(lat, lon))
                {
                    info!(
                        "Using stale weather cache as offline fallback for ({}, {})",
                        lat, lon
                    );
                    return Ok(stale.data);
                }
                // 4. No cache available -> Weather unavailable (Sec 13)
                Err(err)
            }
        }
    }
}

impl Default for WeatherService {
    fn default() -> Self {
        Self::new()
    }
}

/// Periodic weather update stream (every 30 minutes, aligned with cache duration)
pub fn weather_update_stream() -> impl futures::Stream<Item = ()> {
    futures::stream::unfold((), |_| async {
        tokio::time::sleep(tokio::time::Duration::from_secs(1800)).await;
        Some(((), ()))
    })
}
/// State manager ensuring stale asynchronous weather responses are rejected
/// and never overwrite newer state.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WeatherStateManager {
    pub current_generation: u64,
    pub weather: Option<WeatherData>,
    pub weather_error: bool,
}

impl WeatherStateManager {
    pub fn new() -> Self {
        Self {
            current_generation: 0,
            weather: None,
            weather_error: false,
        }
    }

    pub fn with_initial(cached: Option<WeatherData>) -> Self {
        Self {
            current_generation: 1,
            weather: cached,
            weather_error: false,
        }
    }

    pub fn next_generation(&mut self) -> u64 {
        self.current_generation += 1;
        self.current_generation
    }

    pub fn reset_for_location_change(&mut self) -> u64 {
        self.weather = None;
        self.weather_error = false;
        self.next_generation()
    }

    pub fn apply_update(
        &mut self,
        generation: u64,
        result: Result<WeatherData, WeatherError>,
    ) -> bool {
        if generation != self.current_generation {
            return false;
        }
        match result {
            Ok(data) => {
                self.weather = Some(data);
                self.weather_error = false;
            }
            Err(_) => {
                if self.weather.is_none() {
                    self.weather_error = true;
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::weather::provider::{CurrentWeather, DailyForecast};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockProvider {
        call_count: AtomicUsize,
        should_fail: bool,
    }

    #[async_trait::async_trait]
    impl WeatherProvider for MockProvider {
        async fn fetch_weather(&self, _lat: f64, _lon: f64) -> Result<WeatherData, WeatherError> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            if self.should_fail {
                Err(WeatherError::Network("simulated network error".into()))
            } else {
                Ok(WeatherData {
                    current: CurrentWeather {
                        temperature_celsius: 18.5,
                        weather_code: 0,
                        condition_text: "Clear sky".into(),
                    },
                    daily: vec![DailyForecast {
                        date: "2026-09-20".into(),
                        weather_code: 0,
                        condition_text: "Clear sky".into(),
                        temp_min_celsius: 15.0,
                        temp_max_celsius: 25.0,
                    }],
                })
            }
        }
    }

    #[tokio::test]
    async fn test_weather_service_fetch_and_cache() {
        let temp_dir = std::env::temp_dir().join(format!("toodle_test_{}", std::process::id()));
        let temp_cache = temp_dir.join("weather_cache.json");
        let _ = std::fs::remove_file(&temp_cache);

        let mock = Arc::new(MockProvider {
            call_count: AtomicUsize::new(0),
            should_fail: false,
        });
        let service = WeatherService::with_provider_and_cache(mock.clone(), temp_cache.clone());

        let result = service.get_weather(35.68, 139.69).await;
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.current.condition_text, "Clear sky");
        assert_eq!(data.current.temperature_celsius, 18.5);
        assert_eq!(mock.call_count.load(Ordering::SeqCst), 1);

        // Immediate second call for SAME location should use cache and not increment call count
        let result2 = service.get_weather(35.68, 139.69).await;
        assert!(result2.is_ok());
        assert_eq!(mock.call_count.load(Ordering::SeqCst), 1);

        // Different location (e.g. London) must NOT use Tokyo's cache and must fetch fresh
        let result_london = service.get_weather(51.5074, -0.1278).await;
        assert!(result_london.is_ok());
        assert_eq!(mock.call_count.load(Ordering::SeqCst), 2);

        let _ = std::fs::remove_file(&temp_cache);
        let _ = std::fs::remove_dir(&temp_dir);
    }

    #[tokio::test]
    async fn test_weather_service_offline_fallback() {
        let temp_dir =
            std::env::temp_dir().join(format!("toodle_test_fallback_{}", std::process::id()));
        let temp_cache = temp_dir.join("weather_cache.json");
        let _ = std::fs::remove_file(&temp_cache);

        // First, populate cache
        let mock_success = Arc::new(MockProvider {
            call_count: AtomicUsize::new(0),
            should_fail: false,
        });
        let service_success =
            WeatherService::with_provider_and_cache(mock_success, temp_cache.clone());
        let _ = service_success.get_weather(35.68, 139.69).await.unwrap();

        // Now test with failing provider: should fall back to cached data
        let mock_fail = Arc::new(MockProvider {
            call_count: AtomicUsize::new(0),
            should_fail: true,
        });
        let service_fail = WeatherService::with_provider_and_cache(mock_fail, temp_cache.clone());
        let res = service_fail.get_weather(35.68, 139.69).await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap().current.condition_text, "Clear sky");

        let _ = std::fs::remove_file(&temp_cache);
        let _ = std::fs::remove_dir(&temp_dir);
    }

    #[tokio::test]
    async fn test_weather_service_invalid_coordinates() {
        let service = WeatherService::new();
        // Lat out of range
        let err_lat = service.get_weather(95.0, 139.69).await.unwrap_err();
        match err_lat {
            WeatherError::Parse(msg) => assert!(msg.contains("Invalid coordinates"), "{msg}"),
            _ => panic!("Expected WeatherError::Parse"),
        }

        // Lon out of range
        let err_lon = service.get_weather(35.0, 190.0).await.unwrap_err();
        match err_lon {
            WeatherError::Parse(msg) => assert!(msg.contains("Invalid coordinates"), "{msg}"),
            _ => panic!("Expected WeatherError::Parse"),
        }

        // NaN
        let err_nan = service.get_weather(f64::NAN, 0.0).await.unwrap_err();
        match err_nan {
            WeatherError::Parse(msg) => assert!(msg.contains("Invalid coordinates"), "{msg}"),
            _ => panic!("Expected WeatherError::Parse"),
        }
    }

    #[test]
    fn test_stale_weather_generation_rejection() {
        let mut mgr = WeatherStateManager::with_initial(None);
        assert_eq!(mgr.current_generation, 1);

        // Generation 1 initiated for Tokyo
        let gen_tokyo = mgr.next_generation();
        assert_eq!(gen_tokyo, 2);

        // User changes location to London -> Generation 3 initiated
        let gen_london = mgr.reset_for_location_change();
        assert_eq!(gen_london, 3);
        assert!(mgr.weather.is_none());

        let tokyo_data = WeatherData {
            current: CurrentWeather {
                temperature_celsius: 25.0,
                weather_code: 0,
                condition_text: "Clear sky (Tokyo)".into(),
            },
            daily: Vec::new(),
        };

        let london_data = WeatherData {
            current: CurrentWeather {
                temperature_celsius: 14.0,
                weather_code: 61,
                condition_text: "Rain (London)".into(),
            },
            daily: Vec::new(),
        };

        // London response arrives first for Generation 3
        let applied_london = mgr.apply_update(gen_london, Ok(london_data));
        assert!(applied_london);
        assert_eq!(
            mgr.weather.as_ref().unwrap().current.condition_text,
            "Rain (London)"
        );

        // Delayed Tokyo response arrives late for Generation 2
        let applied_tokyo = mgr.apply_update(gen_tokyo, Ok(tokyo_data));
        assert!(!applied_tokyo, "Stale Generation 2 response must be rejected");

        // Weather state must remain London!
        assert_eq!(
            mgr.weather.as_ref().unwrap().current.condition_text,
            "Rain (London)"
        );
        assert_eq!(
            mgr.weather.as_ref().unwrap().current.temperature_celsius,
            14.0
        );
    }
}
