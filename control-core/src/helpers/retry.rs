use std::future::Future;

/// Retries a fallible operation up to `n` times synchronously.
///
/// This function executes the provided closure `f` repeatedly until either:
/// - The operation succeeds (returns `Ok(T)`)
/// - All retry attempts are exhausted
///
/// # Arguments
///
/// * `n` - The maximum number of retry attempts (0 means only one attempt)
/// * `f` - A mutable closure that returns a `Result<T, E>`
///
/// # Returns
///
/// Returns `Ok(T)` if any attempt succeeds, or `Err(E)` containing the error
/// from the last failed attempt if all retries are exhausted.
///
/// # Examples
///
/// ```rust
/// use control_core::helpers::retry::retry_n_times;
///
/// let mut attempt = 0;
/// let result = retry_n_times(3, || {
///     attempt += 1;
///     if attempt < 3 {
///         Err("failed")
///     } else {
///         Ok("success")
///     }
/// });
/// assert_eq!(result, Ok("success"));
/// ```
pub fn retry_n_times<T, E, F>(n: usize, mut f: F) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    let mut last_err: Option<E> = None;
    for _ in 0..=n {
        match f() {
            Ok(value) => return Ok(value),
            Err(err) => last_err = Some(err),
        }
    }
    Err(last_err.unwrap())
}

/// Retries a fallible async operation up to `n` times.
///
/// This function executes the provided async closure `f` repeatedly until either:
/// - The operation succeeds (returns `Ok(T)`)
/// - All retry attempts are exhausted
///
/// Each attempt is awaited before proceeding to the next retry or returning.
///
/// # Arguments
///
/// * `n` - The maximum number of retry attempts (0 means only one attempt)
/// * `f` - A mutable closure that returns a `Future<Output = Result<T, E>>`
///
/// # Returns
///
/// Returns `Ok(T)` if any attempt succeeds, or `Err(E)` containing the error
/// from the last failed attempt if all retries are exhausted.
///
/// # Examples
///
/// ```rust
/// use control_core::helpers::retry::retry_n_times_async;
/// use std::sync::Arc;
/// use std::sync::atomic::{AtomicUsize, Ordering};
///
/// async fn example() {
///     let attempt = Arc::new(AtomicUsize::new(0));
///     let attempt_clone = attempt.clone();
///     let result = retry_n_times_async(3, move || {
///         let attempt = attempt_clone.clone();
///         async move {
///             let current = attempt.fetch_add(1, Ordering::SeqCst) + 1;
///             if current < 3 {
///                 Err("failed")
///             } else {
///                 Ok("success")
///             }
///         }
///     }).await;
///     assert_eq!(result, Ok("success"));
/// }
/// ```
pub async fn retry_n_times_async<T, E, F, Fut>(n: usize, mut f: F) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut last_err: Option<E> = None;
    for _ in 0..=n {
        match f().await {
            Ok(value) => return Ok(value),
            Err(err) => last_err = Some(err),
        }
    }
    Err(last_err.unwrap())
}

/// Retries a fallible operation until a closure decides to stop retrying.
///
/// This function executes the provided closure `f` repeatedly until either:
/// - The operation succeeds (returns `Ok(T)`)
/// - The retry condition returns false
///
/// # Arguments
///
/// * `f` - A mutable closure that returns a `Result<T, E>`
/// * `should_retry` - A closure that takes `&E` and returns `bool` to decide if retry should continue
///
/// # Returns
///
/// Returns `Ok(T)` if any attempt succeeds, or `Err(E)` containing the error
/// from the last failed attempt when retries stop.
///
/// # Examples
///
/// ```rust
/// use control_core::helpers::retry::retry_conditionally;
///
/// let mut attempt = 0;
/// let result = retry_conditionally(
///     || {
///         attempt += 1;
///         if attempt < 3 {
///             Err(format!("failed attempt {}", attempt))
///         } else {
///             Ok("success")
///         }
///     },
///     |err| {
///         // Retry up to 3 times
///         !err.contains("attempt 3")
///     }
/// );
/// assert_eq!(result, Ok("success"));
/// ```
pub fn retry_conditionally<T, E, F, R>(mut f: F, mut should_retry: R) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
    R: FnMut(&E) -> bool,
{
    loop {
        match f() {
            Ok(value) => return Ok(value),
            Err(err) => {
                if !should_retry(&err) {
                    return Err(err);
                }
            }
        }
    }
}

/// Retries a fallible async operation until a closure decides to stop retrying.
///
/// This function executes the provided async closure `f` repeatedly until either:
/// - The operation succeeds (returns `Ok(T)`)
/// - The retry condition returns false
///
/// # Arguments
///
/// * `f` - A mutable closure that returns a `Future<Output = Result<T, E>>`
/// * `should_retry` - A closure that takes `&E` and returns `bool` to decide if retry should continue
///
/// # Returns
///
/// Returns `Ok(T)` if any attempt succeeds, or `Err(E)` containing the error
/// from the last failed attempt when retries stop.
///
/// # Examples
///
/// ```rust
/// use control_core::helpers::retry::retry_conditionally_async;
/// use std::sync::Arc;
/// use std::sync::atomic::{AtomicUsize, Ordering};
///
/// async fn example() {
///     let attempt = Arc::new(AtomicUsize::new(0));
///     let attempt_clone = attempt.clone();
///     let result = retry_conditionally_async(
///         move || {
///             let attempt = attempt_clone.clone();
///             async move {
///                 let current = attempt.fetch_add(1, Ordering::SeqCst) + 1;
///                 if current < 3 {
///                     Err(format!("failed attempt {}", current))
///                 } else {
///                     Ok("success")
///                 }
///             }
///         },
///         |err| {
///             // Retry up to 3 times
///             !err.contains("attempt 3")
///         }
///     ).await;
///     assert_eq!(result, Ok("success"));
/// }
/// ```
pub async fn retry_conditionally_async<T, E, F, Fut, R>(
    mut f: F,
    mut should_retry: R,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    R: FnMut(&E) -> bool,
{
    loop {
        match f().await {
            Ok(value) => return Ok(value),
            Err(err) => {
                if !should_retry(&err) {
                    return Err(err);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    // Simple blocking executor for async functions in tests
    fn block_on<F: std::future::Future>(future: F) -> F::Output {
        use std::sync::Arc;
        use std::task::{Context, Poll, Waker};
        use std::thread;

        struct DummyWaker;
        impl std::task::Wake for DummyWaker {
            fn wake(self: Arc<Self>) {}
        }

        let waker = Waker::from(Arc::new(DummyWaker));
        let mut context = Context::from_waker(&waker);
        let mut future = Box::pin(future);

        loop {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(result) => return result,
                Poll::Pending => {
                    thread::yield_now();
                }
            }
        }
    }

    #[test]
    fn test_retry_success_first_attempt() {
        let result = retry_n_times(3, || Ok::<i32, &str>(42));
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_retry_success_after_failures() {
        let mut attempt = 0;
        let result = retry_n_times(3, || {
            attempt += 1;
            if attempt < 3 { Err("failed") } else { Ok(42) }
        });
        assert_eq!(result, Ok(42));
        assert_eq!(attempt, 3);
    }

    #[test]
    fn test_retry_all_attempts_fail() {
        let mut attempt = 0;
        let result: Result<i32, &str> = retry_n_times(2, || {
            attempt += 1;
            Err("always fails")
        });
        assert_eq!(result, Err("always fails"));
        assert_eq!(attempt, 3); // 0..=2 means 3 attempts
    }

    #[test]
    fn test_retry_zero_retries() {
        let mut attempt = 0;
        let result: Result<i32, &str> = retry_n_times(0, || {
            attempt += 1;
            Err("failed")
        });
        assert_eq!(result, Err("failed"));
        assert_eq!(attempt, 1); // Only one attempt
    }

    #[test]
    fn test_retry_async_success_first_attempt() {
        let result = block_on(retry_n_times_async(3, || async { Ok::<i32, &str>(42) }));
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_retry_async_success_after_failures() {
        let attempt = Rc::new(RefCell::new(0));
        let attempt_clone = attempt.clone();
        let result = block_on(retry_n_times_async(3, move || {
            let attempt = attempt_clone.clone();
            async move {
                let mut a = attempt.borrow_mut();
                *a += 1;
                if *a < 3 { Err("failed") } else { Ok(42) }
            }
        }));
        assert_eq!(result, Ok(42));
        assert_eq!(*attempt.borrow(), 3);
    }

    #[test]
    fn test_retry_async_all_attempts_fail() {
        let attempt = Rc::new(RefCell::new(0));
        let attempt_clone = attempt.clone();
        let result: Result<i32, &str> = block_on(retry_n_times_async(2, move || {
            let attempt = attempt_clone.clone();
            async move {
                let mut a = attempt.borrow_mut();
                *a += 1;
                Err("always fails")
            }
        }));
        assert_eq!(result, Err("always fails"));
        assert_eq!(*attempt.borrow(), 3); // 0..=2 means 3 attempts
    }

    #[test]
    fn test_retry_and_success_with_condition() {
        let mut attempt = 0;
        let result: Result<i32, String> = retry_conditionally(
            || {
                attempt += 1;
                if attempt < 3 {
                    Err(format!("failed attempt {}", attempt))
                } else {
                    Ok(42)
                }
            },
            |err| {
                // Retry as long as it's not the 3rd attempt
                !err.contains("attempt 3")
            },
        );

        assert_eq!(result, Ok(42));
        assert_eq!(attempt, 3);
    }

    #[test]
    fn test_retry_and_stop_early() {
        let mut attempt = 0;
        let result: Result<i32, String> = retry_conditionally(
            || {
                attempt += 1;
                Err(format!("failed attempt {}", attempt))
            },
            |_err| {
                // Stop after first failure
                false
            },
        );

        assert!(result.is_err());
        assert_eq!(attempt, 1);
        assert_eq!(result.unwrap_err(), "failed attempt 1");
    }

    #[test]
    fn test_retry_async_success_with_condition() {
        let attempt = Rc::new(RefCell::new(0));
        let attempt_clone = attempt.clone();
        let result: Result<i32, String> = block_on(retry_conditionally_async(
            move || {
                let attempt = attempt_clone.clone();
                async move {
                    let mut a = attempt.borrow_mut();
                    *a += 1;
                    if *a < 3 {
                        Err(format!("failed attempt {}", *a))
                    } else {
                        Ok(42)
                    }
                }
            },
            |err| {
                // Retry as long as it's not the 3rd attempt
                !err.contains("attempt 3")
            },
        ));

        assert_eq!(result, Ok(42));
        assert_eq!(*attempt.borrow(), 3);
    }

    #[test]
    fn test_retry_async_stop_early() {
        let attempt = Rc::new(RefCell::new(0));
        let attempt_clone = attempt.clone();
        let result: Result<i32, String> = block_on(retry_conditionally_async(
            move || {
                let attempt = attempt_clone.clone();
                async move {
                    let mut a = attempt.borrow_mut();
                    *a += 1;
                    Err(format!("failed attempt {}", *a))
                }
            },
            |_err| {
                // Stop after first failure
                false
            },
        ));

        assert!(result.is_err());
        assert_eq!(*attempt.borrow(), 1);
        assert_eq!(result.unwrap_err(), "failed attempt 1");
    }
}
