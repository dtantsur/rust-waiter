// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Simple waiter trait for synchronous actions.
//!
//! The `Waiter` thread represents some action that can be polled for, and
//! that can also fail.

use async_trait::async_trait;
use tokio::time::sleep;
use tokio::time::{Duration, Instant};

/// Trait representing a waiter for some asynchronous action to finish.
///
/// The type `T` is the final type of the action, `E` is an error.
#[async_trait]
pub trait Waiter<T, E> {
    /// Default timeout for this action.
    ///
    /// This timeout is used in the `wait` method.
    /// If `None, wait forever by default.
    fn default_wait_timeout(&self) -> Option<Duration>;

    /// Default delay between two retries.
    fn default_delay(&self) -> Duration;

    /// Update the current state of the action.
    ///
    /// Returns `T` if the action is finished, `None` if it is not. All errors
    /// are propagated via the `Result`.
    ///
    /// This method should not be called again after it returned the final
    /// result.
    async fn poll(&mut self) -> Result<Option<T>, E>;

    /// Error to return on timeout.
    fn timeout_error(&self) -> E;

    /// Wait for the default amount of time.
    ///
    /// Consumes the `Waiter`.
    /// Returns `OperationTimedOut` if the timeout is reached.
    async fn wait(self) -> Result<T, E>
    where
        Self: Sized,
    {
        let duration = self.default_wait_timeout();
        match duration {
            Some(duration) => self.wait_for(duration).await,
            None => self.wait_forever().await,
        }
    }

    /// Wait for specified amount of time.
    ///
    /// Returns `OperationTimedOut` if the timeout is reached.
    async fn wait_for(self, duration: Duration) -> Result<T, E>
    where
        Self: Sized,
    {
        let delay = self.default_delay();
        self.wait_for_with_delay(duration, delay).await
    }

    /// Wait for specified amount of time.
    async fn wait_for_with_delay(mut self, duration: Duration, delay: Duration) -> Result<T, E>
    where
        Self: Sized,
    {
        let start = Instant::now();
        while Instant::now().duration_since(start) <= duration {
            if let Some(result) = self.poll().await? {
                return Ok(result);
            };
            sleep(delay).await;
        }
        Err(self.timeout_error())
    }

    /// Wait forever.
    async fn wait_forever(self) -> Result<T, E>
    where
        Self: Sized,
    {
        let delay = self.default_delay();
        self.wait_forever_with_delay(delay).await
    }

    /// Wait forever with given delay between attempts.
    async fn wait_forever_with_delay(mut self, delay: Duration) -> Result<T, E>
    where
        Self: Sized,
    {
        loop {
            if let Some(result) = self.poll().await? {
                return Ok(result);
            };
            sleep(delay).await;
        }
    }
}

/// Current state of the waiter.
///
/// Type `T` is the current state of the resource, and does not have to match
/// type `T` of `Waiter<T>`.
pub trait WaiterCurrentState<T> {
    /// Get the current representation of the resource.
    ///
    /// Valid as of the last `poll` call.
    fn waiter_current_state(&self) -> &T;
}
