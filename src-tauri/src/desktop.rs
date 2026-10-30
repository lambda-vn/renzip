//! Handing a file to the desktop when nothing is associated with its type.
//!
//! A .rpyc, and plenty of what an .rpa holds, has no application registered
//! for it. Opening then fails, and the useful answer is to let the user pick
//! a program themselves. Every desktop offers something for that, and no two
//! offer the same thing, so each one gets its own path here and the caller
//! is told which one was taken.

// The only unsafe code of the application: two calls into the Windows shell.
#![allow(unsafe_code)]

use std::path::Path;

/// What the desktop would do with a file, asked before anything is opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Association {
    /// An application is registered for this type and its program is there.
    ///
    /// Only `classify` below ever builds one, and that function is Windows
    /// only: on Linux and macOS `association_of` always answers `Unknown`,
    /// so this variant is never produced there. It stays in the enum
    /// unconditionally because `entry_association` in lib.rs matches all
    /// four variants and the frontend's contract is one string among four
    /// on every platform, not three on Linux and macOS.
    #[allow(dead_code)]
    Runnable,
    /// The registered handler is the desktop's own "open with" picker, which
    /// is how Windows answers for a type nothing claims.
    Picker,
    /// The registered program was named, and it is not on the disk any more.
    /// Opening would go nowhere.
    Dead,
    /// The desktop did not say. Windows answers this for a handler that is a
    /// packaged Store application, which has no plain executable to name and
    /// opens perfectly well, so it must never be taken for a refusal.
    Unknown,
}

/// Ask the desktop what it would open `path` with.
///
/// Windows answers with the executable it would start, which is worth asking
/// because ShellExecute does not report the interesting failures: it returns
/// success for a handler whose program was uninstalled, and it returns
/// success again when it quietly routes the file to its own picker. Asking
/// first is the only way to tell those two apart from a real application.
#[cfg(windows)]
pub fn association_of(path: &Path) -> Association {
    classify(registered_executable(path).as_deref())
}

/// Turn the program Windows named, if it named one, into an answer.
///
/// Split from the call that fetches it so the reasoning can be tested; the
/// system call itself cannot be, since what it returns is whatever the
/// machine happens to have installed.
#[cfg(windows)]
fn classify(executable: Option<&str>) -> Association {
    let Some(executable) = executable else {
        return Association::Unknown;
    };
    let executable = Path::new(executable);

    // Windows hands anything it has no handler for to OpenWith.exe, which is
    // the picker itself rather than an application.
    if executable
        .file_name()
        .is_some_and(|name| name.eq_ignore_ascii_case("openwith.exe"))
    {
        return Association::Picker;
    }
    if executable.is_file() {
        Association::Runnable
    } else {
        Association::Dead
    }
}

/// Path of the program registered for the extension of `path`, if any.
#[cfg(windows)]
fn registered_executable(path: &Path) -> Option<String> {
    use windows_sys::Win32::UI::Shell::{AssocQueryStringW, ASSOCF_NONE, ASSOCSTR_EXECUTABLE};

    let extension = path.extension()?.to_str()?;
    let association: Vec<u16> = format!(".{extension}\0").encode_utf16().collect();
    let verb: Vec<u16> = "open\0".encode_utf16().collect();
    let mut buffer = vec![0u16; 512];
    let mut length = buffer.len() as u32;

    // SAFETY: both input strings are null terminated and outlive the call,
    // and the output buffer is handed over with its real length, which the
    // function writes back into. Nothing here is kept by the callee.
    let result = unsafe {
        AssocQueryStringW(
            ASSOCF_NONE,
            ASSOCSTR_EXECUTABLE,
            association.as_ptr(),
            verb.as_ptr(),
            buffer.as_mut_ptr(),
            &mut length,
        )
    };
    if result != 0 {
        return None;
    }

    // The written length counts the terminator on some versions and not on
    // others, so the string ends where its first null does.
    let end = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    Some(String::from_utf16_lossy(&buffer[..end]))
}

/// Ask the desktop what it would open `path` with.
///
/// Neither Linux nor macOS has a cheap, honest answer to that question: the
/// desktops that could answer it want a D-Bus round trip, and the ones that
/// cannot would have to be guessed at from .desktop files. Trying and looking
/// at the result is what is left.
#[cfg(not(windows))]
pub fn association_of(_path: &Path) -> Association {
    Association::Unknown
}

/// What was offered to the user for a file the desktop could not open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Fallback {
    /// The desktop's own "open with" chooser was shown.
    Chooser,
    /// No chooser exists here; the caller should show the file in its folder.
    None,
}

/// Ask the desktop to let the user choose an application for `path`.
///
/// Never fails loudly: a desktop that has nothing to offer answers
/// [`Fallback::None`], and the caller falls back to revealing the file.
#[cfg(windows)]
pub fn choose_application(path: &Path) -> Fallback {
    use std::os::windows::ffi::OsStrExt;

    use windows_sys::Win32::System::Com::{
        CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
    };
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let verb: Vec<u16> = "openas\0".encode_utf16().collect();
    let file: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // ShellExecuteW can hand the verb to shell extensions, which are COM
    // objects: Microsoft's documentation asks for COM to be initialized on
    // the calling thread first, and this one is a blocking-pool thread that
    // never did. A thread already initialized in another mode answers
    // RPC_E_CHANGED_MODE, a failure, and must then not be uninitialized.
    //
    // SAFETY: the reserved pointer must be null; every successful
    // CoInitializeEx is paired with exactly one CoUninitialize on the same
    // thread, below.
    let com = unsafe {
        CoInitializeEx(
            std::ptr::null(),
            (COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) as u32,
        )
    };

    // SAFETY: both strings are null terminated and live until the call
    // returns, and the four remaining arguments are the documented "no owner
    // window, no parameters, no working directory" nulls. ShellExecuteW does
    // not take ownership of anything it is given.
    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            file.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };

    if com >= 0 {
        // SAFETY: pairs the successful CoInitializeEx above, same thread.
        unsafe { CoUninitialize() };
    }

    // Documented convention of ShellExecuteW: anything above 32 is success,
    // anything below is an error code dressed up as a handle.
    if result as isize > 32 {
        Fallback::Chooser
    } else {
        Fallback::None
    }
}

/// Ask the desktop to let the user choose an application for `path`.
///
/// KDE's KIO opens its own "open with" dialog for a type nothing claims, and
/// is reached through kioclient. Nothing equivalent exists as a plain command
/// on the other Linux desktops: the portal that would do it wants a D-Bus
/// session and a file descriptor passed over it, which is a dependency this
/// application does not carry for one menu entry. Those desktops get the file
/// revealed in their file manager instead.
#[cfg(target_os = "linux")]
pub fn choose_application(path: &Path) -> Fallback {
    use std::process::{Command, Stdio};

    for program in ["kioclient", "kioclient5"] {
        let spawned = Command::new(program)
            .arg("exec")
            .arg(path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        // Not waited on here: the dialog stays up until the user is done
        // with it, and this thread has an interface waiting on it. A thread
        // of its own waits instead, or the exited child stays a zombie
        // until the application quits.
        if let Ok(mut child) = spawned {
            std::thread::spawn(move || child.wait());
            return Fallback::Chooser;
        }
    }
    Fallback::None
}

/// Ask the desktop to let the user choose an application for `path`.
///
/// macOS has no way to raise its "open with" panel for an arbitrary file from
/// outside an application, so the file is revealed in the Finder instead.
#[cfg(target_os = "macos")]
pub fn choose_application(_path: &Path) -> Fallback {
    Fallback::None
}

/// Mark `path` as downloaded from the internet, the way a browser marks what
/// it saves.
///
/// A copy extracted from an archive carries no trace of where the archive
/// came from, so Windows would run a .exe or a .bat out of it without the
/// warning the download itself would have raised. The Zone.Identifier
/// stream, zone 3 (internet), is what SmartScreen, the shell's security
/// prompt and Office's protected view read before opening a file.
///
/// # Errors
///
/// The file system refused the stream, as FAT does: the caller must then
/// not open the file.
#[cfg(windows)]
pub fn mark_downloaded(path: &Path) -> std::io::Result<()> {
    let mut stream = path.as_os_str().to_os_string();
    stream.push(":Zone.Identifier");
    std::fs::write(stream, "[ZoneTransfer]\r\nZoneId=3\r\n")
}

/// Mark `path` as downloaded: nothing to write outside Windows, where
/// [`runs_when_opened`] keeps scripts from being opened instead.
///
/// # Errors
///
/// Never.
#[cfg(not(windows))]
pub fn mark_downloaded(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

/// Whether opening `path` would run it rather than show it, on a desktop
/// that has no mark like Windows' to warn before running a downloaded file.
///
/// Linux and macOS hand a script or a launcher to the program associated
/// with it, which may well be its interpreter, a .jar to Java or a .desktop
/// to the session. Wine, once installed, claims the Windows types too: its
/// executables, shortcuts, scripts, control panel modules and registry
/// files all run under it. Such a file is revealed in its folder instead,
/// where the user decides. Windows answers false: the mark is its warning.
pub fn runs_when_opened(path: &Path) -> bool {
    const RUNNABLE: &[&str] = &[
        "app", "appimage", "bash", "bat", "bin", "cmd", "com", "command", "cpl", "csh", "deb",
        "desktop", "dmg", "exe", "fish", "hta", "jar", "js", "ksh", "lnk", "lua", "msi", "php",
        "pif", "pkg", "pl", "ps1", "py", "pyw", "rb", "reg", "rpm", "run", "scr", "sh", "tool",
        "vbe", "vbs", "wsf", "zsh",
    ];
    !cfg!(windows)
        && path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| RUNNABLE.contains(&extension.to_ascii_lowercase().as_str()))
}

#[cfg(all(test, windows))]
mod tests {
    use super::{classify, Association};

    #[test]
    fn a_program_that_is_gone_is_the_only_dead_association() {
        let here = std::env::current_exe().expect("this test binary exists");
        let mine = here.to_str().expect("its path is text");

        assert_eq!(classify(Some(mine)), Association::Runnable);
        assert_eq!(
            classify(Some("C:\\Program Files\\Nothing\\gone.exe")),
            Association::Dead
        );
        assert_eq!(
            classify(Some("C:\\WINDOWS\\system32\\OpenWith.exe")),
            Association::Picker
        );
        assert_eq!(
            classify(Some("C:\\WINDOWS\\system32\\openwith.EXE")),
            Association::Picker,
            "the shell does not care about case and neither should this"
        );
        // A packaged application has no executable to name, and answering
        // Dead there would send every .png to the picker.
        assert_eq!(classify(None), Association::Unknown);
    }
}

#[cfg(test)]
mod script_tests {
    use std::path::Path;

    use super::runs_when_opened;

    #[test]
    fn scripts_are_kept_from_running_outside_windows() {
        // Windows warns through the mark instead, so it never answers true.
        assert_eq!(runs_when_opened(Path::new("game/Patch.SH")), !cfg!(windows));
        assert_eq!(runs_when_opened(Path::new("mods/tool.jar")), !cfg!(windows));
        // Run by Wine where it is installed.
        assert_eq!(
            runs_when_opened(Path::new("game/Setup.scr")),
            !cfg!(windows)
        );
        assert!(!runs_when_opened(Path::new("images/bg.png")));
        assert!(!runs_when_opened(Path::new("README")));
    }
}
