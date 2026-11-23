use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_MANIFEST_URL: &str = "https://sekonic.dijios.com/downloads/agent-version.json";
pub const DEFAULT_DOWNLOAD_URL: &str = "https://sekonic.dijios.com/downloads/sekonic-agent.exe";
pub const WEB_APP_URL: &str = "https://sekonic.dijios.com/";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentManifest {
    pub version: String,
    #[serde(default = "default_download_url")]
    pub download_url: String,
    #[serde(default)]
    pub notes: String,
}

fn default_download_url() -> String {
    DEFAULT_DOWNLOAD_URL.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(default = "default_true")]
    pub auto_update: bool,
}

fn default_true() -> bool {
    true
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self { auto_update: true }
    }
}

pub fn config_file_path() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("APPDATA"))
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("SekonicAgent").join("config.json")
}

pub fn load_config() -> AgentConfig {
    let path = config_file_path();
    let Ok(raw) = fs::read_to_string(&path) else {
        return AgentConfig::default();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

pub fn save_config(cfg: &AgentConfig) {
    let path = config_file_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(cfg) {
        let _ = fs::write(&path, json);
    }
}

/// Parses `major.minor.patch` (ignoring any leading `v` or `-prerelease` suffix) and returns true if `latest > current`.
pub fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse = |s: &str| -> (u64, u64, u64) {
        let trimmed = s.trim().trim_start_matches('v').trim_start_matches('V');
        let core = trimmed.split(['-', '+']).next().unwrap_or(trimmed);
        let mut parts = core.split('.');
        let maj = parts.next().and_then(|p| p.parse::<u64>().ok()).unwrap_or(0);
        let min = parts.next().and_then(|p| p.parse::<u64>().ok()).unwrap_or(0);
        let pat = parts.next().and_then(|p| p.parse::<u64>().ok()).unwrap_or(0);
        (maj, min, pat)
    };
    parse(latest) > parse(current)
}

#[derive(Debug, Clone)]
pub struct UpdateCheckResult {
    pub current_version: String,
    pub latest_version: String,
    pub download_url: String,
    pub update_available: bool,
}

pub fn check_for_update_blocking(manifest_url: Option<&str>) -> Result<UpdateCheckResult, String> {
    let url = manifest_url.unwrap_or(DEFAULT_MANIFEST_URL);
    let current = env!("CARGO_PKG_VERSION").to_string();

    // Add cache-buster query param
    let sep = if url.contains('?') { '&' } else { '?' };
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let busted_url = format!("{url}{sep}t={ts}");

    let body = http_get_bytes(&busted_url)?;
    let manifest: AgentManifest = serde_json::from_slice(&body)
        .map_err(|e| format!("Sürüm manifesti ayrıştırılamadı: {e}"))?;

    let dl_url = if manifest.download_url.trim().is_empty() {
        DEFAULT_DOWNLOAD_URL.to_string()
    } else {
        manifest.download_url.trim().to_string()
    };

    let available = is_newer_version(&manifest.version, &current);
    Ok(UpdateCheckResult {
        current_version: current,
        latest_version: manifest.version,
        download_url: dl_url,
        update_available: available,
    })
}

/// Downloads the new executable to `%TEMP%`, verifies PE `MZ` header, writes a hidden `.cmd` updater script,
/// spawns it in the background, and returns the staged path (caller should exit immediately so the script can replace the exe).
pub fn download_and_spawn_updater(download_url: &str) -> Result<PathBuf, String> {
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Mevcut çalıştırılabilir dosya yolu alınamadı: {e}"))?;
    let pid = std::process::id();

    let sep = if download_url.contains('?') { '&' } else { '?' };
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let busted_url = format!("{download_url}{sep}t={ts}");

    let bytes = http_get_bytes(&busted_url)?;
    if bytes.len() < 100_000 || &bytes[0..2] != b"MZ" {
        return Err("İndirilen ajan dosyası geçerli bir Windows çalıştırılabilir dosyası (.exe) değil.".into());
    }

    let temp_dir = std::env::temp_dir();
    let staged_exe = temp_dir.join(format!("sekonic-agent-update-{pid}.exe"));
    fs::write(&staged_exe, &bytes)
        .map_err(|e| format!("Güncelleme dosyası geçici klasöre yazılamadı: {e}"))?;

    spawn_background_updater(&staged_exe, &current_exe, pid)?;
    Ok(staged_exe)
}

#[cfg(windows)]
fn spawn_background_updater(staged_exe: &Path, current_exe: &Path, pid: u32) -> Result<(), String> {
    use std::os::windows::process::CommandExt;

    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join(format!("sekonic-agent-updater-{pid}.cmd"));

    let src_str = staged_exe.to_string_lossy();
    let dst_str = current_exe.to_string_lossy();

    let script_content = format!(
        "@echo off\r\n\
         chcp 65001 >NUL 2>&1\r\n\
         setlocal\r\n\
         set \"PID={pid}\"\r\n\
         set \"SRC={src_str}\"\r\n\
         set \"DST={dst_str}\"\r\n\
         for /L %%I in (1,1,20) do (\r\n\
           tasklist /FI \"PID eq %PID%\" 2>NUL | find /I \"%PID%\" >NUL\r\n\
           if errorlevel 1 goto :replace\r\n\
           ping -n 2 127.0.0.1 >NUL\r\n\
         )\r\n\
         taskkill /F /PID %PID% >NUL 2>&1\r\n\
         ping -n 2 127.0.0.1 >NUL\r\n\
         :replace\r\n\
         for /L %%R in (1,1,20) do (\r\n\
           copy /Y \"%SRC%\" \"%DST%\" >NUL 2>&1\r\n\
           if not errorlevel 1 goto :launch\r\n\
           ping -n 2 127.0.0.1 >NUL\r\n\
         )\r\n\
         goto :cleanup\r\n\
         :launch\r\n\
         del /F /Q \"%SRC%\" >NUL 2>&1\r\n\
         start \"\" \"%DST%\"\r\n\
         :cleanup\r\n\
         del /F /Q \"%~f0\" >NUL 2>&1\r\n"
    );

    fs::write(&script_path, script_content.as_bytes())
        .map_err(|e| format!("Güncelleme scripti oluşturulamadı: {e}"))?;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    std::process::Command::new("cmd.exe")
        .args(["/C", &script_path.to_string_lossy()])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| format!("Arka plan güncelleme scripti başlatılamadı: {e}"))?;

    Ok(())
}

#[cfg(not(windows))]
fn spawn_background_updater(_staged_exe: &Path, _current_exe: &Path, _pid: u32) -> Result<(), String> {
    Err("Otomatik güncelleme scripti yalnızca Windows üzerinde desteklenir.".into())
}

#[cfg(windows)]
fn http_get_bytes(url: &str) -> Result<Vec<u8>, String> {
    use std::ffi::c_void;
    use std::ptr::null;

    type Hinternet = *mut c_void;
    const WINHTTP_ACCESS_TYPE_DEFAULT_PROXY: u32 = 0;
    const WINHTTP_FLAG_SECURE: u32 = 0x0080_0000;
    const WINHTTP_OPTION_SECURE_PROTOCOLS: u32 = 84;
    const WINHTTP_FLAG_SECURE_PROTOCOL_ALL_TLS: u32 = 0x0000_0A80; // TLS 1.0 | 1.1 | 1.2
    const WINHTTP_QUERY_STATUS_CODE: u32 = 19;
    const WINHTTP_QUERY_FLAG_NUMBER: u32 = 0x2000_0000;

    #[link(name = "winhttp")]
    extern "system" {
        fn WinHttpOpen(
            psz_agent: *const u16,
            dw_access_type: u32,
            psz_proxy: *const u16,
            psz_proxy_bypass: *const u16,
            dw_flags: u32,
        ) -> Hinternet;
        fn WinHttpSetOption(
            h_internet: Hinternet,
            dw_option: u32,
            lp_buffer: *const c_void,
            dw_buffer_length: u32,
        ) -> i32;
        fn WinHttpConnect(
            h_session: Hinternet,
            pswz_server_name: *const u16,
            n_server_port: u16,
            dw_reserved: u32,
        ) -> Hinternet;
        fn WinHttpOpenRequest(
            h_connect: Hinternet,
            pwsz_verb: *const u16,
            pwsz_object_name: *const u16,
            pwsz_version: *const u16,
            pwsz_referrer: *const u16,
            ppwsz_accept_types: *const *const u16,
            dw_flags: u32,
        ) -> Hinternet;
        fn WinHttpSendRequest(
            h_request: Hinternet,
            lpsz_headers: *const u16,
            dw_headers_length: u32,
            lp_optional: *mut c_void,
            dw_optional_length: u32,
            dw_total_length: u32,
            dw_context: usize,
        ) -> i32;
        fn WinHttpReceiveResponse(h_request: Hinternet, lp_reserved: *mut c_void) -> i32;
        fn WinHttpQueryHeaders(
            h_request: Hinternet,
            dw_info_level: u32,
            pwsz_name: *const u16,
            lp_buffer: *mut c_void,
            lpdw_buffer_length: *mut u32,
            lpdw_index: *mut u32,
        ) -> i32;
        fn WinHttpQueryDataAvailable(h_request: Hinternet, lpdw_number_of_bytes_available: *mut u32) -> i32;
        fn WinHttpReadData(
            h_request: Hinternet,
            lp_buffer: *mut c_void,
            dw_number_of_bytes_to_read: u32,
            lpdw_number_of_bytes_read: *mut u32,
        ) -> i32;
        fn WinHttpCloseHandle(h_internet: Hinternet) -> i32;
    }

    let (is_https, host, port, path) = parse_http_url(url)?;
    let to_wide = |s: &str| -> Vec<u16> { s.encode_utf16().chain(std::iter::once(0)).collect() };

    let agent_w = to_wide("SekonicAgent/1.0");
    let host_w = to_wide(&host);
    let path_w = to_wide(&path);
    let verb_w = to_wide("GET");

    unsafe {
        let session = WinHttpOpen(
            agent_w.as_ptr(),
            WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
            null(),
            null(),
            0,
        );
        if session.is_null() {
            return Err("WinHttpOpen başarısız.".into());
        }

        let mut tls_flags = WINHTTP_FLAG_SECURE_PROTOCOL_ALL_TLS;
        let _ = WinHttpSetOption(
            session,
            WINHTTP_OPTION_SECURE_PROTOCOLS,
            (&raw mut tls_flags).cast(),
            std::mem::size_of::<u32>() as u32,
        );

        let connect = WinHttpConnect(session, host_w.as_ptr(), port, 0);
        if connect.is_null() {
            WinHttpCloseHandle(session);
            return Err(format!("Sunucuya bağlanılamadı: {host}:{port}"));
        }

        let flags = if is_https { WINHTTP_FLAG_SECURE } else { 0 };
        let request = WinHttpOpenRequest(
            connect,
            verb_w.as_ptr(),
            path_w.as_ptr(),
            null(),
            null(),
            null(),
            flags,
        );
        if request.is_null() {
            WinHttpCloseHandle(connect);
            WinHttpCloseHandle(session);
            return Err("WinHttpOpenRequest başarısız.".into());
        }

        let ok = WinHttpSendRequest(request, null(), 0, std::ptr::null_mut(), 0, 0, 0) != 0
            && WinHttpReceiveResponse(request, std::ptr::null_mut()) != 0;

        if !ok {
            WinHttpCloseHandle(request);
            WinHttpCloseHandle(connect);
            WinHttpCloseHandle(session);
            return Err(format!("HTTP isteği başarısız: {url}"));
        }

        let mut status_code: u32 = 0;
        let mut status_len = std::mem::size_of::<u32>() as u32;
        if WinHttpQueryHeaders(
            request,
            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            null(),
            (&raw mut status_code).cast(),
            &raw mut status_len,
            std::ptr::null_mut(),
        ) != 0
            && status_code >= 400
        {
            WinHttpCloseHandle(request);
            WinHttpCloseHandle(connect);
            WinHttpCloseHandle(session);
            return Err(format!("Sunucu HTTP {status_code} döndürdü: {url}"));
        }

        let mut out = Vec::new();
        loop {
            let mut avail: u32 = 0;
            if WinHttpQueryDataAvailable(request, &raw mut avail) == 0 || avail == 0 {
                break;
            }
            let mut buf = vec![0u8; avail as usize];
            let mut read: u32 = 0;
            if WinHttpReadData(request, buf.as_mut_ptr().cast(), avail, &raw mut read) == 0 || read == 0 {
                break;
            }
            out.extend_from_slice(&buf[..read as usize]);
        }

        WinHttpCloseHandle(request);
        WinHttpCloseHandle(connect);
        WinHttpCloseHandle(session);

        Ok(out)
    }
}

#[cfg(not(windows))]
fn http_get_bytes(_url: &str) -> Result<Vec<u8>, String> {
    Err("WinHTTP yalnızca Windows üzerinde kullanılabilir.".into())
}

fn parse_http_url(url: &str) -> Result<(bool, String, u16, String), String> {
    let (is_https, rest) = if let Some(r) = url.strip_prefix("https://") {
        (true, r)
    } else if let Some(r) = url.strip_prefix("http://") {
        (false, r)
    } else {
        return Err(format!("Desteklenmeyen URL şeması: {url}"));
    };

    let (authority, path) = match rest.find('/') {
        Some(idx) => (&rest[..idx], &rest[idx..]),
        None => (rest, "/"),
    };

    let (host, port) = match authority.rsplit_once(':') {
        Some((h, p)) => {
            let port_num = p.parse::<u16>().map_err(|_| format!("Geçersiz port: {p}"))?;
            (h.to_string(), port_num)
        }
        None => (authority.to_string(), if is_https { 443 } else { 80 }),
    };

    Ok((is_https, host, port, path.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semver_comparison_works() {
        assert!(is_newer_version("0.2.9", "0.2.8"));
        assert!(is_newer_version("v0.3.0", "0.2.9"));
        assert!(is_newer_version("1.0.0", "0.9.9"));
        assert!(!is_newer_version("0.2.8", "0.2.8"));
        assert!(!is_newer_version("0.2.7", "0.2.8"));
    }

    #[test]
    fn url_parsing_works() {
        let (https, host, port, path) =
            parse_http_url("https://sekonic.dijios.com/downloads/agent-version.json?t=1").unwrap();
        assert!(https);
        assert_eq!(host, "sekonic.dijios.com");
        assert_eq!(port, 443);
        assert_eq!(path, "/downloads/agent-version.json?t=1");
    }
}

