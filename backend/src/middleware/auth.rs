use crate::config::Config;
use crate::models::Claims;
use crate::utils::jwt::validate_token;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::{ok, Ready, LocalBoxFuture};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Auth;

impl<S, B> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddleware {
            service: Rc::new(RefCell::new(service)),
        })
    }
}

pub struct AuthMiddleware<S> {
    service: Rc<RefCell<S>>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
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
        
        Box::pin(async move {
            let auth_header = req.headers().get("Authorization");
            
            if let Some(auth_header) = auth_header {
                let config = req.app_data::<actix_web::web::Data<Config>>()
                    .ok_or_else(|| actix_web::error::ErrorInternalServerError("Config not found"))?;
                
                if let Ok(auth_str) = auth_header.to_str() {
                    let token = auth_str.strip_prefix("Bearer ").unwrap_or(auth_str);
                    
                    if let Ok(claims) = validate_token(token, config) {
                        req.extensions_mut().insert(claims);
                        return service.borrow_mut().call(req).await;
                    }
                }
            }
            
            Err(actix_web::error::ErrorUnauthorized(
                serde_json::json!({
                    "error": "unauthorized",
                    "message": "Authentication required. Please provide a valid token."
                })
            ))
        })
    }
}

pub struct OptionalAuth;

impl<S, B> Transform<S, ServiceRequest> for OptionalAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = OptionalAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(OptionalAuthMiddleware {
            service: Rc::new(RefCell::new(service)),
        })
    }
}

pub struct OptionalAuthMiddleware<S> {
    service: Rc<RefCell<S>>,
}

impl<S, B> Service<ServiceRequest> for OptionalAuthMiddleware<S>
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
        
        Box::pin(async move {
            if let Some(auth_header) = req.headers().get("Authorization") {
                let config = req.app_data::<actix_web::web::Data<Config>>()
                    .ok_or_else(|| actix_web::error::ErrorInternalServerError("Config not found"))?;
                
                if let Ok(auth_str) = auth_header.to_str() {
                    let token = auth_str.strip_prefix("Bearer ").unwrap_or(auth_str);
                    
                    if let Ok(claims) = validate_token(token, config) {
                        req.extensions_mut().insert(claims);
                    }
                }
            }
            
            service.borrow_mut().call(req).await
        })
    }
}
