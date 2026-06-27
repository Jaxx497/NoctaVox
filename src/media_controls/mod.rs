use anyhow::anyhow;
use crossbeam_channel::{Receiver, bounded};
use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig,
};
use std::time::Duration;

pub struct MediaControlsHandle {
    controls: MediaControls,
    event_rx: Receiver<MediaControlEvent>,
}

impl MediaControlsHandle {
    pub fn new() -> anyhow::Result<Self> {
        let (event_tx, event_rx) = bounded::<MediaControlEvent>(32);

        #[cfg(not(target_os = "windows"))]
        let config = PlatformConfig {
            dbus_name: "noctavox",
            display_name: "Noctavox",
            hwnd: None,
        };

        #[cfg(target_os = "windows")]
        let config = PlatformConfig {
            dbus_name: "noctavox",
            display_name: "Noctavox",
            hwnd: Some(
                create_hidden_window()
                    .ok_or_else(|| anyhow!("Failed to create hidden window for SMTC"))?,
            ),
        };

        let mut controls = MediaControls::new(config)
            .map_err(|e| anyhow!("Failed to create OS media controls: {e:?}"))?;

        controls
            .attach(move |event: MediaControlEvent| {
                let _ = event_tx.try_send(event);
            })
            .map_err(|e| anyhow!("Failed to attach media controls handler: {e:?}"))?;

        Ok(Self { controls, event_rx })
    }

    pub fn event_rx(&self) -> &Receiver<MediaControlEvent> {
        &self.event_rx
    }

    pub fn update_metadata(&mut self, title: &str, artist: &str, album: &str, duration: Duration) {
        let _ = self.controls.set_metadata(MediaMetadata {
            title: Some(title),
            artist: Some(artist),
            album: Some(album),
            duration: Some(duration),
            cover_url: None,
        });
    }

    pub fn set_playing(&mut self, elapsed: Duration) {
        let _ = self.controls.set_playback(MediaPlayback::Playing {
            progress: Some(MediaPosition(elapsed)),
        });
    }

    pub fn set_paused(&mut self, elapsed: Duration) {
        let _ = self.controls.set_playback(MediaPlayback::Paused {
            progress: Some(MediaPosition(elapsed)),
        });
    }

    pub fn set_stopped(&mut self) {
        let _ = self.controls.set_playback(MediaPlayback::Stopped);
    }
}

/// Create a zero-size, invisible top-level window owned by this process.
///
/// SMTC's `ISystemMediaTransportControlsInterop::GetForWindow` has two requirements:
///   1. The HWND must belong to the calling process  (message-only windows → E_ACCESSDENIED)
///   2. The HWND must be a real top-level window     (HWND_MESSAGE parent  → E_INVALIDARG)
///
/// This function creates a proper Win32 window with:
///   - Desktop as parent  
///   - Width/height = 0 [completely invisible]
///   - No WS_VISIBLE [never shown]    
///   - WS_EX_TOOLWINDOW [excluded from taskbar and Alt-Tab]
///   - WS_EX_NOACTIVATE [cannot be focused]
#[cfg(target_os = "windows")]
fn create_hidden_window() -> Option<*mut std::ffi::c_void> {
    use windows_sys::Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, RegisterClassExW, WNDCLASSEXW, WS_EX_NOACTIVATE,
            WS_EX_TOOLWINDOW,
        },
    };

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }

    // Null-terminated UTF-16 class name (replaces the `windows` crate's `w!` macro).
    let class_name: Vec<u16> = "NoctaVoxSTMC\0".encode_utf16().collect();

    unsafe {
        let h_instance = GetModuleHandleW(std::ptr::null());

        // Default::default() zero-initialises all fields (null handles, zero styles).
        // Only the four non-zero fields need to be set explicitly.
        RegisterClassExW(&WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wnd_proc),
            hInstance: h_instance,
            lpszClassName: class_name.as_ptr(),
            ..Default::default()
        });

        // null parent → desktop → proper top-level window (NOT HWND_MESSAGE).
        // dwStyle = 0, size 0×0, no WS_VISIBLE → completely invisible.
        let hwnd = CreateWindowExW(
            WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
            class_name.as_ptr(),
            std::ptr::null(),
            0,
            0,
            0,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            h_instance,
            std::ptr::null(),
        );

        (!hwnd.is_null()).then_some(hwnd)
    }
}
