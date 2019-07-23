use std::sync::{Arc,RwLock};
use std::time::{Duration,Instant};
use chrono::{DateTime,NaiveDateTime,Utc};
use std::collections::HashMap;

use tracing::*;

#[derive(Debug)]
struct RateLimitState{
	limit: usize,
	remaining: usize,
	reset_at: DateTime<Utc>,
}

impl RateLimitState{
	pub fn get_remaining(&self) -> Option<usize>{
		if self.reset_at <= Utc::now() {
			return None;
		}
		Some(self.remaining)
	}
	pub fn use_one(&mut self) -> bool{
        if self.reset_at <= Utc::now() {
            //assume we will be granted some rate limit not that reset is passed
			return true;
		}
		match self.remaining {
            0 => false,
			_other => {
                self.remaining -= 1;
		        true
            }
		}
	}

    pub fn time_to_reset(&self) -> Duration{
        Duration::from_secs((self.reset_at.timestamp()-Utc::now().timestamp()) as u64)
    }

    pub fn parse(headers: &hyper::HeaderMap) -> Result<Option<Self>,failure::Error>
    {
        let (limit,remaining,reset_at) = match (headers.get("X-RateLimit-Limit"),headers.get("X-RateLimit-Remaining"),headers.get("X-RateLimit-Reset")){
            (Some(limit),Some(remaining),Some(reset_at)) => (limit,remaining,reset_at),
            _otherwise => {
                //Assume it's (None,None,None) as other combinations don't really make sesnse anyway
                return Ok(None);
            }
        };

        let limit = usize::from_str_radix(limit.to_str()?,10)?;
        let remaining = usize::from_str_radix(remaining.to_str()?,10)?;
        let reset_at = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(i64::from_str_radix(reset_at.to_str()?,10)?,0),Utc);

        Ok(Some(RateLimitState{
            limit,
            remaining,
            reset_at
        }))
    }
}

#[derive(Clone,Default)]
pub struct RateLimiter{
    rate_limits: Arc<RwLock<HashMap<String,RateLimitState>>>
}

impl RateLimiter{
    //TODO: this might have performance issues. investigate fine-grained locking
	pub async fn enforce_limit<'a>(&'a self,endpoint: &'a str) -> Result<(),tokio::timer::Error>{
        loop{
            //get any required wait first, then run it once the rate_limits guard is dropped to avoid lock contention
            let wait = {
                let guard = self.rate_limits.read().expect("Rate limits poisoned");
                let wait_endpoint = if let Some(rate_limit) = guard.get(endpoint){
                    if rate_limit.get_remaining() == Some(0){
                        Some(Instant::now() + rate_limit.time_to_reset())
                    }else{
                        None
                    }
                }else{
                    None
                };
                let wait_global = if let Some(rate_limit) = guard.get("GLOBAL"){
                    if rate_limit.get_remaining() == Some(0){
                        Some(Instant::now() + rate_limit.time_to_reset())
                    }else{
                        None
                    }
                }else{
                    None
                };
                debug!("Rate limit state: {:?}",self.rate_limits.read());
                std::cmp::max(wait_endpoint,wait_global)
            };
            if let Some(wait) = wait{
                trace!("Waiting on ratelimit: {:?}",wait);
                tokio::timer::Delay::new(wait).await;
                trace!("Rate limit wait complete");
            }
            if self.use_resource(endpoint){
                return Ok(());
            }
            //something else used the new allocation of resource before we could.
            //we should ideally implement some sort of non-linear backoff & jittering here to avoid contention
        }
	}

    fn use_resource(&self, endpoint: &str) -> bool{
        let mut guard = self.rate_limits.write().expect("Rate limits poisoned");
        let endpoint_allowed = if let Some(rate_limit) = guard.get_mut(endpoint){
            rate_limit.use_one()
        }else{
            true
        };
        let global_allowed = if let Some(rate_limit) = guard.get_mut("GLOBAL"){
            rate_limit.use_one()
        }else{
            true
        };
        endpoint_allowed&&global_allowed
    }

    pub fn update_limits(&self, endpoint: String, headers: &hyper::HeaderMap){
        let global = headers.contains_key("X-RateLimit-Global");

        let rate_limit = match RateLimitState::parse(headers){
            Ok(Some(rate_limit)) => rate_limit,
            Ok(None) => {
                //nothing to update
                return;
            }
            Err(e) => {
                //Not fatal, so only warn
                warn!("Could not parse ratelimits: {:?}",e);
                return;
            }
        };

        let entry = if global {
             "GLOBAL".into()
        }else{
            endpoint
        };

        self.rate_limits.write().unwrap().insert(entry,rate_limit);
    }
}