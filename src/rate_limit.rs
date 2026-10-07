use axum::extract::{ConnectInfo, Request, State};
use axum::http::{HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use governor::clock::{Clock, DefaultClock, QuantaClock, Reference};
use governor::middleware::NoOpMiddleware;
use governor::state::keyed::DashMapStateStore;
use governor::{Quota, RateLimiter};
use std::net::{IpAddr, SocketAddr};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub type IpLimiter = RateLimiter<IpAddr, DashMapStateStore<IpAddr>, DefaultClock, NoOpMiddleware>;

fn build_quota(rate_limit: u32, window_seconds: u64) -> Quota {
    Quota::with_period(
        Duration::from_secs(window_seconds)
    ).unwrap()
        .allow_burst(
            NonZeroU32::new(rate_limit).expect("rate_limit must be > 0")
        )
}

pub fn build_limiters(config: &RateLimitConfig) -> (Arc<IpLimiter>, Arc<IpLimiter>, Arc<IpLimiter>) {
    let shorten = Arc::new(RateLimiter::keyed(
        build_quota(config.shorten_limit, config.shorten_window_seconds))
    );

    let redirect = Arc::new(RateLimiter::keyed(
        build_quota(config.redirect_limit, config.redirect_window_seconds))
    );

    let health = Arc::new(RateLimiter::keyed(
        build_quota(config.health_limit, config.health_window_seconds))
    );

    (shorten, redirect, health)
}

#[derive(Clone)]
pub struct RateLimitConfig {
    pub shorten_limit: u32,
    pub shorten_window_seconds: u64,
    pub redirect_limit: u32,
    pub redirect_window_seconds: u64,
    pub health_limit: u32,
    pub health_window_seconds: u64,
}

impl RateLimitConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let shorten_limit = std::env::var("RATE_LIMIT_SHORTEN")?.trim().parse()?;
        let shorten_window_seconds = std::env::var("RATE_LIMIT_SHORTEN_WINDOW_SECS")?
            .trim().parse()?;

        let redirect_limit = std::env::var("RATE_LIMIT_REDIRECT")?.trim().parse()?;
        let redirect_window_seconds = std::env::var("RATE_LIMIT_REDIRECT_WINDOW_SECS")?
            .trim().parse()?;

        let health_limit = std::env::var("RATE_LIMIT_HEALTH")?.trim().parse()?;
        let health_window_seconds = std::env::var("RATE_LIMIT_HEALTH_WINDOW_SECS")?
            .trim().parse()?;

        Ok(
            Self {
                shorten_limit,
                shorten_window_seconds,
                redirect_limit,
                redirect_window_seconds,
                health_limit,
                health_window_seconds
            }
        )
    }
}

#[derive(Clone)]
pub struct RouteLimit {
    pub limiter: Arc<IpLimiter>,
    pub limit: u32,
    pub window_seconds: u64,
}

pub async fn rate_limit_middleware(
    State(RouteLimit{ limiter, limit, window_seconds }): State<RouteLimit>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let ip = addr.ip();

    match limiter.check_key(&ip) {
        Ok(_) => {
            let mut resp = next.run(request).await;
            let headers = resp.headers_mut();
            headers.insert(
                "X-RateLimit-Limit",
                HeaderValue::from_str(&limit.to_string()).unwrap()
            );
            let reset_secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + window_seconds;
            headers.insert(
                "X-RateLimit-Reset",
                HeaderValue::from_str(&reset_secs.to_string()).unwrap()
            );
            Ok(resp)
        }
        Err(negative) => {
            let earliest = negative.earliest_possible();
            let now = QuantaClock::default().now();
            let retry_after = Duration::from(earliest.duration_since(now));

            let mut resp = (StatusCode::TOO_MANY_REQUESTS, "Rate Limit Exceeded").into_response();
            let headers = resp.headers_mut();
            headers.insert(
                "X-RateLimit-Limit",
                HeaderValue::from_str(&limit.to_string()).unwrap()
            );
            headers.insert(
                "Retry-After",
                HeaderValue::from_str(&retry_after.as_secs().to_string()).unwrap()
            );
            let reset_secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + retry_after.as_secs();
            headers.insert(
                "X-RateLimit-Reset",
                HeaderValue::from_str(&reset_secs.to_string()).unwrap()
            );

            Err(resp)
        }
    }
}
