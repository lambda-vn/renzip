//! Sanitizing of archive entry paths.
//!
//! Entry paths come out of the archive index, which is untrusted input: a
//! crafted archive may hold "../../etc/passwd", a Windows drive letter, a
//! reserved device name, or characters no Windows file system accepts.
//! Nothing here touches the file system; the final containment check against
//! the destination directory lives in [`is_contained`] and is called by
//! [`crate::archive`] just before writing.

use std::path::{Component, Path, PathBuf};

/// Why an entry path was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathRejection {
    /// The path is empty or holds only separators and dots.
    Empty,
    /// The path starts at a file system root.
    Absolute,
    /// The path carries a Windows drive letter, as in "C:foo".
    DriveLetter,
    /// The path holds a ".." component.
    Traversal,
    /// A component is a Windows reserved device name (CON, NUL, COM1...).
    ReservedName(String),
    /// A component holds a character no Windows path may contain.
    ForbiddenCharacter(char),
    /// A component ends with a dot or a space, which Windows silently strips.
    TrailingDotOrSpace(String),
}

impl std::fmt::Display for PathRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathRejection::Empty => write!(f, "empty path"),
            PathRejection::Absolute => write!(f, "absolute path"),
            PathRejection::DriveLetter => write!(f, "drive letter"),
            PathRejection::Traversal => write!(f, "path traversal"),
            PathRejection::ReservedName(name) => write!(f, "reserved name '{name}'"),
            PathRejection::ForbiddenCharacter(c) => write!(f, "forbidden character {c:?}"),
            PathRejection::TrailingDotOrSpace(name) => {
                write!(f, "component '{name}' ends with a dot or a space")
            }
        }
    }
}

/// Windows device names, reserved with or without an extension.
const RESERVED_STEMS: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Characters a path component may never hold once separators are normalized.
const FORBIDDEN: [char; 8] = ['<', '>', ':', '"', '|', '?', '*', '\\'];

/// Check an archive entry path and turn it into a relative [`PathBuf`].
///
/// Both separators are accepted on input, since archives written on Windows
/// do contain backslashes; the result always uses the platform separator.
///
/// # Errors
///
/// Returns the reason the path was refused. A refused entry is skipped and
/// reported, never extracted under a repaired name: silently rewriting a
/// hostile path hides the fact that the archive tried something.
pub fn sanitize(raw: &str) -> std::result::Result<PathBuf, PathRejection> {
    let normalized = raw.replace('\\', "/");

    if normalized.starts_with('/') {
        return Err(PathRejection::Absolute);
    }
    // A drive-relative path such as "C:foo" has no leading slash, so it has to
    // be caught on the raw string rather than on the components below.
    if normalized.len() >= 2 && normalized.as_bytes()[1] == b':' {
        return Err(PathRejection::DriveLetter);
    }

    let mut out = PathBuf::new();
    for segment in normalized.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            return Err(PathRejection::Traversal);
        }
        check_component(segment)?;
        out.push(segment);
    }

    if out.as_os_str().is_empty() {
        return Err(PathRejection::Empty);
    }
    // Belt and braces: the loop above cannot emit anything but normal
    // components, and one extra pass over a short path is cheap insurance
    // against a future edit of that loop.
    for component in out.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(PathRejection::Traversal);
        }
    }
    Ok(out)
}

/// Check a single path component for characters and names Windows refuses.
fn check_component(segment: &str) -> std::result::Result<(), PathRejection> {
    for c in segment.chars() {
        if FORBIDDEN.contains(&c) || (c as u32) < 0x20 {
            return Err(PathRejection::ForbiddenCharacter(c));
        }
    }
    if segment.ends_with('.') || segment.ends_with(' ') {
        return Err(PathRejection::TrailingDotOrSpace(segment.to_string()));
    }

    let stem = segment.split('.').next().unwrap_or(segment);
    if RESERVED_STEMS.contains(&stem.to_ascii_uppercase().as_str()) {
        return Err(PathRejection::ReservedName(segment.to_string()));
    }
    Ok(())
}

/// Check that `candidate` really sits inside `root` once both are canonical.
///
/// String checks alone are not enough: a symbolic link or a junction placed
/// inside the destination directory can point anywhere, so the last word goes
/// to the canonical form of the directory that is about to be written to.
///
/// Both paths must exist, so callers create the parent directory first and
/// pass it here, then write the file name inside it.
pub fn is_contained(root: &Path, candidate: &Path) -> std::io::Result<bool> {
    let root = root.canonicalize()?;
    let candidate = candidate.canonicalize()?;
    Ok(candidate.starts_with(&root))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_ordinary_paths() {
        assert_eq!(
            sanitize("game/images/bg.png").unwrap(),
            PathBuf::from("game").join("images").join("bg.png")
        );
        assert_eq!(sanitize("script.rpy").unwrap(), PathBuf::from("script.rpy"));
        assert_eq!(
            sanitize("a//b/./c.txt").unwrap(),
            PathBuf::from("a").join("b").join("c.txt")
        );
        assert_eq!(
            sanitize("dir\\file.rpy").unwrap(),
            PathBuf::from("dir").join("file.rpy")
        );
    }

    #[test]
    fn rejects_traversal_and_absolute_paths() {
        assert_eq!(sanitize("../../etc/passwd"), Err(PathRejection::Traversal));
        assert_eq!(sanitize("game/../../x"), Err(PathRejection::Traversal));
        assert_eq!(sanitize("/etc/passwd"), Err(PathRejection::Absolute));
        assert_eq!(
            sanitize("\\windows\\system32\\a.dll"),
            Err(PathRejection::Absolute)
        );
        assert_eq!(
            sanitize("C:\\Windows\\System32\\a.dll"),
            Err(PathRejection::DriveLetter)
        );
        assert_eq!(sanitize("C:foo"), Err(PathRejection::DriveLetter));
    }

    #[test]
    fn rejects_empty_and_dot_only_paths() {
        assert_eq!(sanitize(""), Err(PathRejection::Empty));
        assert_eq!(sanitize("."), Err(PathRejection::Empty));
        assert_eq!(sanitize("./././"), Err(PathRejection::Empty));
    }

    #[test]
    fn rejects_windows_reserved_names_and_characters() {
        assert_eq!(
            sanitize("CON"),
            Err(PathRejection::ReservedName("CON".into()))
        );
        assert_eq!(
            sanitize("game/nul.txt"),
            Err(PathRejection::ReservedName("nul.txt".into()))
        );
        assert_eq!(
            sanitize("com1.rpy"),
            Err(PathRejection::ReservedName("com1.rpy".into()))
        );
        assert_eq!(
            sanitize("a<b.png"),
            Err(PathRejection::ForbiddenCharacter('<'))
        );
        assert_eq!(
            sanitize("a\u{1}b.png"),
            Err(PathRejection::ForbiddenCharacter('\u{1}'))
        );
        assert!(matches!(
            sanitize("trailing./x"),
            Err(PathRejection::TrailingDotOrSpace(_))
        ));
    }

    #[test]
    fn console_is_not_reserved_but_con_is() {
        assert!(sanitize("console/log.txt").is_ok());
        assert!(sanitize("con.txt").is_err());
    }
}
