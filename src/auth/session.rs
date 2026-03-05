/// Thin wrapper around localStorage for caching auth state across reloads.
/// Privy manages the real token lifecycle; this just lets us show the correct
/// UI instantly before the async Privy init completes.
use web_sys::window;

const SESSION_KEY: &str = "privy_session_active";
const USER_ID_KEY: &str = "privy_user_id";

pub fn save_session(user_id: &str) {
    if let Some(storage) = local_storage() {
        let _ = storage.set_item(SESSION_KEY, "1");
        let _ = storage.set_item(USER_ID_KEY, user_id);
    }
}

pub fn clear_session() {
    if let Some(storage) = local_storage() {
        let _ = storage.remove_item(SESSION_KEY);
        let _ = storage.remove_item(USER_ID_KEY);
    }
}

/// Returns true if we previously saved a session.
/// Privy will still re-validate the real token asynchronously.
pub fn has_cached_session() -> bool {
    local_storage()
        .and_then(|s| s.get_item(SESSION_KEY).ok().flatten())
        .map(|v| v == "1")
        .unwrap_or(false)
}

pub fn cached_user_id() -> Option<String> {
    local_storage().and_then(|s| s.get_item(USER_ID_KEY).ok().flatten())
}

fn local_storage() -> Option<web_sys::Storage> {
    window()?.local_storage().ok()?
}