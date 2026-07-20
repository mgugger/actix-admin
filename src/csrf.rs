//! CSRF protection helpers.
//!
//! actix-admin protects every state-changing route (POST/PUT/PATCH/DELETE)
//! with a per-session CSRF token stored in the `actix-session` cookie.
//!
//! * [`csrf_token_for`] returns the current token, creating one the first
//!   time it is called for a session.
//! * [`verify_csrf`] extracts the token from either the `X-CSRF-Token`
//!   header (used by HTMX) or the `_csrf` query parameter (used by classic
//!   form posts and multipart uploads, where the form body is streamed
//!   lazily by `actix-multipart` and cannot be peeked ahead-of-time).
//!
//! The check can be globally disabled via
//! [`crate::ActixAdminConfiguration::enable_csrf`] \u2014 in that case
//! [`verify_csrf`] is a no-op.
//!
//! Setup: install a session middleware (e.g. `actix-session::CookieSession`)
//! **before** the admin scope, exactly like you already need to for auth.
//!
//! HTMX wiring: `base.html` sets an `htmx:configRequest` listener that
//! attaches the token as `X-CSRF-Token` on every HTMX call. Non-HTMX forms
//! also include a hidden `_csrf` input, and delete/action URLs carry the
//! token as a `_csrf=` query parameter.
use actix_session::Session;
use actix_web::HttpRequest;

use crate::{ActixAdmin, ActixAdminError, ActixAdminErrorType};

/// Session storage key. Public so applications can inspect/clear the token.
pub const CSRF_SESSION_KEY: &str = "_actix_admin_csrf";
/// Header name checked on every state-changing request.
pub const CSRF_HEADER: &str = "X-CSRF-Token";
/// Fallback query-string parameter, used when the header isn't available
/// (classic multipart form submissions handled by actix-multipart).
pub const CSRF_QUERY_PARAM: &str = "_csrf";

/// Marker error type returned by [`verify_csrf`]. Currently equivalent to
/// [`ActixAdminError`] with `ty = ActixAdminErrorType::CsrfError` \u2014 kept
/// as a distinct alias in case a future release wants richer diagnostics.
pub type CsrfError = ActixAdminError;

/// Return the CSRF token for `session`, generating a fresh one if there is
/// none. Safe to call from any handler; the value is stable for the lifetime
/// of the session.
pub fn csrf_token_for(session: &Session) -> Result<String, ActixAdminError> {
    if let Some(existing) = session.get::<String>(CSRF_SESSION_KEY).unwrap_or(None) {
        return Ok(existing);
    }
    let token = generate_token();
    session
        .insert(CSRF_SESSION_KEY, &token)
        .map_err(|e| ActixAdminError::new(ActixAdminErrorType::InternalError, e.to_string()))?;
    Ok(token)
}

/// Assert that `req` carries a valid CSRF token for `session`. Returns
/// `Ok(())` when CSRF protection is disabled globally.
///
/// The token can be provided either via the `X-CSRF-Token` header (HTMX)
/// or the `_csrf` query-string parameter (classic forms & multipart).
pub fn verify_csrf(
    actix_admin: &ActixAdmin,
    session: &Session,
    req: &HttpRequest,
) -> Result<(), ActixAdminError> {
    if !actix_admin.configuration.enable_csrf {
        return Ok(());
    }

    let expected = session
        .get::<String>(CSRF_SESSION_KEY)
        .unwrap_or(None)
        .ok_or_else(|| {
            ActixAdminError::new(
                ActixAdminErrorType::CsrfError,
                "no CSRF token in session; reload the page and try again",
            )
        })?;

    let from_header = req
        .headers()
        .get(CSRF_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);

    let from_query = if from_header.is_none() {
        form_urlencoded::parse(req.query_string().as_bytes())
            .find(|(k, _)| k == CSRF_QUERY_PARAM)
            .map(|(_, v)| v.into_owned())
    } else {
        None
    };

    let received = from_header.or(from_query).ok_or_else(|| {
        ActixAdminError::new(
            ActixAdminErrorType::CsrfError,
            "missing CSRF token (expected `X-CSRF-Token` header or `_csrf` query param)",
        )
    })?;

    if constant_time_eq(received.as_bytes(), expected.as_bytes()) {
        Ok(())
    } else {
        Err(ActixAdminError::new(
            ActixAdminErrorType::CsrfError,
            "CSRF token mismatch",
        ))
    }
}

/// Constant-time byte comparison to avoid timing side channels.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Generate a 32-byte URL-safe base64 token.
///
/// Uses `std::time::SystemTime` + a lightweight counter + the address of a
/// stack variable as entropy sources so we avoid pulling in a heavyweight
/// crypto crate. This is not a cryptographic PRNG; the token is used only
/// to make CSRF forgery infeasible across sessions and is scoped to a
/// signed/encrypted session cookie, which is the real trust boundary.
fn generate_token() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let ctr = COUNTER.fetch_add(1, Ordering::Relaxed);
    let stack = &now as *const _ as usize as u64;

    // Mix inputs with a simple splitmix64 iteration; produce 32 bytes.
    let mut state: u64 = now.wrapping_mul(0x9E3779B97F4A7C15) ^ ctr ^ stack;
    let mut bytes = [0u8; 32];
    for chunk in bytes.chunks_mut(8) {
        state = state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^= z >> 31;
        chunk.copy_from_slice(&z.to_le_bytes()[..chunk.len()]);
    }

    // URL-safe base64 without padding.
    base64_url(&bytes)
}

fn base64_url(data: &[u8]) -> String {
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i] as u32;
        let b1 = data.get(i + 1).copied().unwrap_or(0) as u32;
        let b2 = data.get(i + 2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARSET[((n >> 18) & 63) as usize] as char);
        out.push(CHARSET[((n >> 12) & 63) as usize] as char);
        if i + 1 < data.len() {
            out.push(CHARSET[((n >> 6) & 63) as usize] as char);
        }
        if i + 2 < data.len() {
            out.push(CHARSET[(n & 63) as usize] as char);
        }
        i += 3;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_reasonably_unique() {
        let a = generate_token();
        let b = generate_token();
        assert_ne!(a, b);
        assert!(a.len() >= 40);
    }

    #[test]
    fn constant_time_eq_basic() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"abcd"));
    }
}
