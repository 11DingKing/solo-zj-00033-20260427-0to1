use crate::config::Config;
use crate::models::Claims;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse, HttpMessage,
};
use futures::future::{ok, Ready, LocalBoxFuture};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use chrono::{Utc, Duration};
use uuid::Uuid;

pub struct RateLimitMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimitService {
            service: Rc::new(RefCell::new(service)),
            rate_limiter: Arc::new(RateLimiter::new(60)),
        })
    }
}

pub struct RateLimitService<S> {
    service: Rc<RefCell<S>>,
    rate_limiter: Arc<RateLimiter>,
}

struct RateLimiter {
    requests: std::sync::Mutex<HashMap<String, (i32, chrono::DateTime<Utc>)>>,
    max_requests: i32,
}

impl RateLimiter {
    fn new(max_requests: i32) -> Self {
        RateLimiter {
            requests: std::sync::Mutex::new(HashMap::new()),
            max_requests,
        }
    }

    fn check_rate_limit(&self, key: &str) -> bool {
        let mut requests = self.requests.lock().unwrap();
        let now = Utc::now();
        let window_start = now - Duration::minutes(1);

        let entry = requests.entry(key.to_string()).or_insert((0, window_start));
        
        if entry.1 < window_start {
            *entry = (1, now);
            true
        } else if entry.0 < self.max_requests {
            entry.0 += 1;
            true
        } else {
            false
        }
    }
}

impl<S, B> Service<ServiceRequest> for RateLimitService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let rate_limiter = self.rate_limiter.clone();

        Box::pin(async move {
            let user_id = req.extensions().get::<Claims>().map(|c| c.sub);
            let ip = req.connection_info().peer_addr().unwrap_or("unknown").to_string();
            
            let key = if let Some(uid) = user_id {
                uid.to_string()
            } else {
                ip
            };

            if !rate_limiter.check_rate_limit(&key) {
                return Err(actix_web::error::ErrorTooManyRequests(
                    serde_json::json!({
                        "error": "rate_limit_exceeded",
                        "message": "Rate limit exceeded. Please wait a minute before trying again.",
                        "retry_after": 60
                    })
                ));
            }

            service.borrow_mut().call(req).await
        })
    }
}
