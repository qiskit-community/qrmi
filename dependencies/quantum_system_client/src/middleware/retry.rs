//
// (C) Copyright IBM 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

//! A drop-in replacement for `reqwest_retry::RetryTransientMiddleware` that
//! doesn't erase the underlying error on the way out.
//!
//! `reqwest_retry::RetryTransientMiddleware` -- even when it ends up
//! retrying zero times -- unconditionally rewraps whatever error escapes
//! `next.run()` as an opaque `RetryError` before returning it, and that
//! rewrapping doesn't preserve a `source()` chain. That means any
//! structured information downstream code (e.g.
//! [`crate::error::QuantumSystemError::from_middleware_error`]) might want
//! to recover from the original error -- including which
//! `reqwest_middleware::Error` variant it actually was -- is unrecoverable
//! by the time it gets there.
//!
//! This middleware reuses the exact same retry *decision* logic (the
//! `RetryPolicy` and `RetryableStrategy` traits, so retry behavior doesn't
//! change at all) but, once it's done retrying, hands back whatever
//! `next.run()` produced completely unchanged instead of wrapping it.

use http::Extensions;
use reqwest::{Request, Response};
use reqwest_middleware::{Error, Middleware, Next, Result};
use reqwest_retry::{DefaultRetryableStrategy, Retryable, RetryableStrategy};
use retry_policies::{RetryDecision, RetryPolicy};
use std::time::{Duration, SystemTime};

/// Retries a request according to `retry_policy`/`retryable_strategy`, same
/// as [`reqwest_retry::RetryTransientMiddleware`], but returns the final
/// result exactly as received -- see the module docs for why.
pub(crate) struct TransparentRetryMiddleware<T, R = DefaultRetryableStrategy>
where
    T: RetryPolicy + Send + Sync + 'static,
    R: RetryableStrategy + Send + Sync + 'static,
{
    retry_policy: T,
    retryable_strategy: R,
}

impl<T> TransparentRetryMiddleware<T, DefaultRetryableStrategy>
where
    T: RetryPolicy + Send + Sync + 'static,
{
    pub(crate) fn new_with_policy(retry_policy: T) -> Self {
        Self::new_with_policy_and_strategy(retry_policy, DefaultRetryableStrategy)
    }
}

impl<T, R> TransparentRetryMiddleware<T, R>
where
    T: RetryPolicy + Send + Sync + 'static,
    R: RetryableStrategy + Send + Sync + 'static,
{
    pub(crate) fn new_with_policy_and_strategy(retry_policy: T, retryable_strategy: R) -> Self {
        Self {
            retry_policy,
            retryable_strategy,
        }
    }
}

#[async_trait::async_trait]
impl<T, R> Middleware for TransparentRetryMiddleware<T, R>
where
    T: RetryPolicy + Send + Sync + 'static,
    R: RetryableStrategy + Send + Sync + 'static,
{
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        let mut n_past_retries = 0;
        let start_time = SystemTime::now();
        loop {
            // Cloning the request object before-the-fact is not ideal, but
            // if the body is a static/shared buffer (e.g. `Bytes`) it's
            // effectively free -- same tradeoff `reqwest_retry` makes.
            let duplicate_request = req.try_clone().ok_or_else(|| {
                Error::Middleware(anyhow::anyhow!(
                    "Request object is not cloneable. Are you passing a streaming body?"
                ))
            })?;

            let result = next.clone().run(duplicate_request, extensions).await;

            if let Some(Retryable::Transient) = self.retryable_strategy.handle(&result) {
                let retry_decision = self.retry_policy.should_retry(start_time, n_past_retries);
                if let RetryDecision::Retry { execute_after } = retry_decision {
                    let duration = execute_after
                        .duration_since(SystemTime::now())
                        .unwrap_or_else(|_| Duration::default());
                    tokio::time::sleep(duration).await;
                    n_past_retries += 1;
                    continue;
                }
            }

            // Unlike `reqwest_retry::RetryTransientMiddleware`, return
            // exactly what `next.run()` produced -- no `RetryError`
            // rewrapping, so the original `reqwest_middleware::Error`
            // variant (and whatever's inside a `Middleware` variant, such
            // as an IAM token-acquisition failure) survives intact for
            // whoever classifies it downstream.
            return result;
        }
    }
}
