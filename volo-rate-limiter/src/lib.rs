#![doc = include_str!("../README.md")]
#![feature(doc_cfg)]
#![feature(associated_type_bounds)]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

mod rate_limit_error;
pub use rate_limit_error::RateLimitError;

mod rate_limiter;
pub use rate_limiter::*;

mod rate_limiter_service;
pub use rate_limiter_service::{RateLimiterLayer, RateLimiterService};
