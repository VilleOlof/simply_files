pub mod private;
pub mod public;
pub mod websocket;

/// A path cannot be root or go back or anything foul
fn path_is_valid(path: impl AsRef<std::path::Path>) -> bool {
    let mut components = path.as_ref().components().peekable();

    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return false;
        }
    }

    return true;
}
