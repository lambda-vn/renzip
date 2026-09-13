//! Turning an entry into something the window can display.
//!
//! Previewing is what makes RenZip useful next to a command line unpacker, so
//! it covers the two things a translator or a modder actually looks at: the
//! text of a script and the pixels of an image. Everything else is announced
//! as binary rather than dumped.

use base64::Engine;
use serde::Serialize;

/// Largest entry shown as text.
const TEXT_LIMIT: u64 = 1024 * 1024;

/// Largest entry shown as an image.
///
/// The bytes travel to the window as base64 inside the IPC message, which
/// costs a third more than the file itself; a background at 8 MiB is already
/// far past what a preview needs.
const IMAGE_LIMIT: u64 = 8 * 1024 * 1024;

/// What the window should render for an entry.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Preview {
    /// Decoded text.
    Text {
        /// Full content of the entry.
        content: String,
    },
    /// An image, ready to be used as the source of an img element.
    Image {
        /// data: URL holding the whole image.
        data_url: String,
    },
    /// Nothing to show, with the reason why.
    Binary {
        /// Size of the entry.
        size: u64,
        /// Why it is not being displayed.
        reason: BinaryReason,
    },
}

/// Why an entry is announced as binary rather than displayed.
///
/// A code rather than a sentence, so the window decides what to say about
/// each case, and in which language, without reading English text.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum BinaryReason {
    /// Larger than a preview of its kind is allowed to read.
    TooLarge {
        /// The limit it went over, in bytes.
        limit: u64,
    },
    /// Neither an image nor UTF-8 text.
    NotText,
}

/// Refuse an entry that is too large before reading it, if it is.
pub fn too_big(path: &str, size: u64) -> Option<Preview> {
    let limit = if mime_of(path).is_some() {
        IMAGE_LIMIT
    } else {
        TEXT_LIMIT
    };
    (size > limit).then_some(Preview::Binary {
        size,
        reason: BinaryReason::TooLarge { limit },
    })
}

/// Decide how to show an entry that has been read.
pub fn of(path: &str, bytes: Vec<u8>) -> Preview {
    let size = bytes.len() as u64;

    if let Some(mime) = mime_of(path) {
        let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
        return Preview::Image {
            data_url: format!("data:{mime};base64,{encoded}"),
        };
    }

    match String::from_utf8(bytes) {
        Ok(content) if !content.contains('\u{0}') => Preview::Text { content },
        _ => Preview::Binary {
            size,
            reason: BinaryReason::NotText,
        },
    }
}

/// Image media type of a path, by extension.
fn mime_of(path: &str) -> Option<&'static str> {
    let extension = path.rsplit_once('.')?.1.to_ascii_lowercase();
    match extension.as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        "bmp" => Some("image/bmp"),
        "ico" => Some("image/x-icon"),
        "svg" => Some("image/svg+xml"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_is_decoded_and_binary_is_named() {
        assert!(matches!(
            of("script.rpy", b"label start:\n".to_vec()),
            Preview::Text { .. }
        ));
        assert!(matches!(
            of("data.bin", vec![0x00, 0x01, 0x02]),
            Preview::Binary {
                reason: BinaryReason::NotText,
                ..
            }
        ));
        assert!(matches!(
            of("images/bg.png", vec![0x89, b'P', b'N', b'G']),
            Preview::Image { .. }
        ));
    }

    #[test]
    fn limits_depend_on_what_the_entry_is() {
        assert!(too_big("script.rpy", TEXT_LIMIT + 1).is_some());
        assert!(too_big("bg.png", TEXT_LIMIT + 1).is_none());
        assert!(too_big("bg.png", IMAGE_LIMIT + 1).is_some());
    }

    #[test]
    fn a_refusal_is_a_code_with_the_limit_it_went_over() {
        let Some(Preview::Binary { reason, .. }) = too_big("script.rpy", TEXT_LIMIT + 1) else {
            panic!("a text entry over the limit is refused");
        };
        assert_eq!(reason, BinaryReason::TooLarge { limit: TEXT_LIMIT });
        assert_eq!(
            serde_json::to_value(&reason).unwrap(),
            serde_json::json!({"code": "too_large", "limit": TEXT_LIMIT})
        );
    }
}
