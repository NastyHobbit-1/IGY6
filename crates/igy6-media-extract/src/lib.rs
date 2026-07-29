//! Local media text extraction for IGY6.
//!
//! Calls tools installed with the product (pdftotext, tesseract, ffmpeg, whisper).
//! Extracted text is returned to the caller for storage inside IGY6 only.
//! Processed proprietary output never leaves the program via this crate.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaExtractResult {
    pub text: String,
    pub method: String,
    pub mime_type: String,
    pub tool: String,
    pub success: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolAvailability {
    pub pdftotext: bool,
    pub tesseract: bool,
    pub ffmpeg: bool,
    pub whisper: bool,
}

pub fn tool_availability() -> ToolAvailability {
    ToolAvailability {
        pdftotext: command_exists("pdftotext"),
        tesseract: command_exists("tesseract"),
        ffmpeg: command_exists("ffmpeg"),
        whisper: command_exists("whisper") || command_exists("whisper-cli") || command_exists("whisper.cpp"),
    }
}

fn command_exists(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .output()
        .map(|output| output.status.success() || !output.stderr.is_empty() || !output.stdout.is_empty())
        .unwrap_or_else(|_| {
            Command::new(name)
                .arg("-version")
                .output()
                .map(|output| output.status.success() || !output.stderr.is_empty() || !output.stdout.is_empty())
                .unwrap_or(false)
        })
}

fn classify_media(mime: &str, filename: &str) -> &'static str {
    let mime = mime.to_lowercase();
    let name = filename.to_lowercase();
    if mime.contains("pdf") || name.ends_with(".pdf") {
        return "pdf";
    }
    if mime.starts_with("image/")
        || name.ends_with(".png")
        || name.ends_with(".jpg")
        || name.ends_with(".jpeg")
        || name.ends_with(".webp")
        || name.ends_with(".tif")
        || name.ends_with(".tiff")
        || name.ends_with(".bmp")
        || name.ends_with(".gif")
    {
        return "image";
    }
    if mime.starts_with("audio/")
        || name.ends_with(".wav")
        || name.ends_with(".mp3")
        || name.ends_with(".m4a")
        || name.ends_with(".ogg")
        || name.ends_with(".flac")
        || name.ends_with(".aac")
    {
        return "audio";
    }
    if mime.starts_with("video/")
        || name.ends_with(".mp4")
        || name.ends_with(".mov")
        || name.ends_with(".mkv")
        || name.ends_with(".webm")
        || name.ends_with(".avi")
    {
        return "video";
    }
    "unknown"
}

fn temp_work_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("igy6-media-extract-{nanos}"));
    let _ = fs::create_dir_all(&dir);
    dir
}

fn write_temp_file(dir: &Path, filename: &str, bytes: &[u8]) -> Result<PathBuf, String> {
    let path = dir.join(filename);
    fs::write(&path, bytes).map_err(|e| format!("failed to write temp media file: {e}"))?;
    Ok(path)
}

fn run_capture(cmd: &mut Command) -> Result<(String, String, bool), String> {
    let output = cmd
        .output()
        .map_err(|e| format!("failed to spawn tool: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok((stdout, stderr, output.status.success()))
}

fn extract_pdf(path: &Path) -> MediaExtractResult {
    if !command_exists("pdftotext") {
        return MediaExtractResult {
            text: String::new(),
            method: "pdf_text_layer".to_string(),
            mime_type: "application/pdf".to_string(),
            tool: "pdftotext".to_string(),
            success: false,
            detail: "pdftotext not installed; run product install / rebuild worker image".to_string(),
        };
    }
    let mut cmd = Command::new("pdftotext");
    cmd.arg("-layout").arg(path.as_os_str()).arg("-");
    match run_capture(&mut cmd) {
        Ok((stdout, stderr, ok)) if ok && !stdout.trim().is_empty() => MediaExtractResult {
            text: stdout,
            method: "pdf_text_layer".to_string(),
            mime_type: "application/pdf".to_string(),
            tool: "pdftotext".to_string(),
            success: true,
            detail: "extracted PDF text layer".to_string(),
        },
        Ok((stdout, stderr, ok)) => {
            // OCR fallback page-by-page is heavy; report honestly and try tesseract on rendered page if available later.
            let detail = if !ok {
                format!("pdftotext failed: {}", stderr.chars().take(400).collect::<String>())
            } else if stdout.trim().is_empty() {
                "PDF has no extractable text layer; image-only PDF may need OCR of rendered pages".to_string()
            } else {
                "pdftotext returned content".to_string()
            };
            MediaExtractResult {
                text: stdout,
                method: "pdf_text_layer".to_string(),
                mime_type: "application/pdf".to_string(),
                tool: "pdftotext".to_string(),
                success: !detail.contains("failed") && !detail.contains("no extractable"),
                detail,
            }
        }
        Err(error) => MediaExtractResult {
            text: String::new(),
            method: "pdf_text_layer".to_string(),
            mime_type: "application/pdf".to_string(),
            tool: "pdftotext".to_string(),
            success: false,
            detail: error,
        },
    }
}

fn extract_image_ocr(path: &Path, mime: &str) -> MediaExtractResult {
    if !command_exists("tesseract") {
        return MediaExtractResult {
            text: String::new(),
            method: "image_ocr".to_string(),
            mime_type: mime.to_string(),
            tool: "tesseract".to_string(),
            success: false,
            detail: "tesseract not installed; run product install / rebuild worker image".to_string(),
        };
    }
    let mut cmd = Command::new("tesseract");
    cmd.arg(path.as_os_str()).arg("stdout").arg("-l").arg("eng");
    match run_capture(&mut cmd) {
        Ok((stdout, stderr, ok)) if ok => MediaExtractResult {
            text: stdout,
            method: "image_ocr".to_string(),
            mime_type: mime.to_string(),
            tool: "tesseract".to_string(),
            success: true,
            detail: if stdout.trim().is_empty() {
                "OCR completed with empty text".to_string()
            } else {
                "OCR completed".to_string()
            },
        },
        Ok((_, stderr, _)) => MediaExtractResult {
            text: String::new(),
            method: "image_ocr".to_string(),
            mime_type: mime.to_string(),
            tool: "tesseract".to_string(),
            success: false,
            detail: format!("tesseract failed: {}", stderr.chars().take(400).collect::<String>()),
        },
        Err(error) => MediaExtractResult {
            text: String::new(),
            method: "image_ocr".to_string(),
            mime_type: mime.to_string(),
            tool: "tesseract".to_string(),
            success: false,
            detail: error,
        },
    }
}

fn ffmpeg_to_wav(input: &Path, wav_out: &Path) -> Result<(), String> {
    if !command_exists("ffmpeg") {
        return Err("ffmpeg not installed; run product install / rebuild worker image".to_string());
    }
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y")
        .arg("-i")
        .arg(input.as_os_str())
        .arg("-vn")
        .arg("-ac")
        .arg("1")
        .arg("-ar")
        .arg("16000")
        .arg(wav_out.as_os_str());
    let (stdout, stderr, ok) = run_capture(&mut cmd)?;
    if ok {
        Ok(())
    } else {
        Err(format!(
            "ffmpeg failed: {} {}",
            stdout.chars().take(200).collect::<String>(),
            stderr.chars().take(300).collect::<String>()
        ))
    }
}

fn whisper_transcribe(wav: &Path) -> Result<String, String> {
    let candidates = ["whisper", "whisper-cli", "whisper.cpp"];
    for name in candidates {
        if !command_exists(name) {
            continue;
        }
        let mut cmd = Command::new(name);
        // Common CLI shapes; first successful parse wins.
        cmd.arg(wav.as_os_str()).arg("--output_format").arg("txt");
        if let Ok((stdout, stderr, ok)) = run_capture(&mut cmd) {
            if ok && !stdout.trim().is_empty() {
                return Ok(stdout);
            }
            if ok {
                // Some builds write sidecar .txt next to input.
                let sidecar = wav.with_extension("txt");
                if let Ok(text) = fs::read_to_string(&sidecar) {
                    if !text.trim().is_empty() {
                        return Ok(text);
                    }
                }
            }
            if !ok && !stderr.is_empty() {
                // try next candidate
                continue;
            }
        }
    }
    Err("no local whisper binary produced transcript (install whisper or whisper.cpp)".to_string())
}

fn extract_av(path: &Path, mime: &str, kind: &str) -> MediaExtractResult {
    let work = temp_work_dir();
    let wav = work.join("audio16k.wav");
    match ffmpeg_to_wav(path, &wav) {
        Err(error) => MediaExtractResult {
            text: String::new(),
            method: format!("{kind}_transcription"),
            mime_type: mime.to_string(),
            tool: "ffmpeg".to_string(),
            success: false,
            detail: error,
        },
        Ok(()) => match whisper_transcribe(&wav) {
            Ok(text) => MediaExtractResult {
                text,
                method: format!("{kind}_transcription"),
                mime_type: mime.to_string(),
                tool: "ffmpeg+whisper".to_string(),
                success: true,
                detail: format!("{kind} transcription completed"),
            },
            Err(error) => MediaExtractResult {
                text: String::new(),
                method: format!("{kind}_transcription"),
                mime_type: mime.to_string(),
                tool: "ffmpeg+whisper".to_string(),
                success: false,
                detail: error,
            },
        },
    }
}

/// Extract searchable text from media bytes using local tools only.
/// Results are for IGY6 internal storage; this crate does not transmit data externally.
pub fn extract_text_from_media(
    bytes: &[u8],
    mime_type: &str,
    filename: &str,
) -> MediaExtractResult {
    let kind = classify_media(mime_type, filename);
    if kind == "unknown" {
        // Fall back: if bytes are valid UTF-8 text, return them.
        if let Ok(text) = std::str::from_utf8(bytes) {
            return MediaExtractResult {
                text: text.replace("\r\n", "\n").replace('\r', "\n"),
                method: "utf8_passthrough".to_string(),
                mime_type: mime_type.to_string(),
                tool: "none".to_string(),
                success: true,
                detail: "content treated as UTF-8 text".to_string(),
            };
        }
        return MediaExtractResult {
            text: String::new(),
            method: "unsupported".to_string(),
            mime_type: mime_type.to_string(),
            tool: "none".to_string(),
            success: false,
            detail: format!("unsupported media type for extraction: {mime_type} / {filename}"),
        };
    }

    let work = temp_work_dir();
    let safe_name = if filename.trim().is_empty() {
        match kind {
            "pdf" => "input.pdf",
            "image" => "input.png",
            "audio" => "input.audio",
            "video" => "input.video",
            _ => "input.bin",
        }
    } else {
        filename
    };
    let path = match write_temp_file(&work, safe_name, bytes) {
        Ok(path) => path,
        Err(error) => {
            return MediaExtractResult {
                text: String::new(),
                method: kind.to_string(),
                mime_type: mime_type.to_string(),
                tool: "none".to_string(),
                success: false,
                detail: error,
            };
        }
    };

    let result = match kind {
        "pdf" => extract_pdf(&path),
        "image" => extract_image_ocr(&path, mime_type),
        "audio" => extract_av(&path, mime_type, "audio"),
        "video" => extract_av(&path, mime_type, "video"),
        _ => MediaExtractResult {
            text: String::new(),
            method: "unsupported".to_string(),
            mime_type: mime_type.to_string(),
            tool: "none".to_string(),
            success: false,
            detail: "unsupported kind".to_string(),
        },
    };

    let _ = fs::remove_dir_all(&work);
    result
}

/// Convenience: prefer media extraction, else UTF-8 normalization.
pub fn extract_or_utf8(bytes: &[u8], mime_type: Option<&str>, filename: Option<&str>) -> MediaExtractResult {
    let mime = mime_type.unwrap_or("application/octet-stream");
    let name = filename.unwrap_or("upload.bin");
    let kind = classify_media(mime, name);
    if kind != "unknown" {
        return extract_text_from_media(bytes, mime, name);
    }
    match std::str::from_utf8(bytes) {
        Ok(text) => MediaExtractResult {
            text: text.replace("\r\n", "\n").replace('\r', "\n"),
            method: "utf8_passthrough".to_string(),
            mime_type: mime.to_string(),
            tool: "none".to_string(),
            success: true,
            detail: "UTF-8 text".to_string(),
        },
        Err(_) => {
            let lossy = String::from_utf8_lossy(bytes).to_string();
            MediaExtractResult {
                text: lossy.replace("\r\n", "\n").replace('\r', "\n"),
                method: "utf8_lossy".to_string(),
                mime_type: mime.to_string(),
                tool: "none".to_string(),
                success: true,
                detail: "lossy UTF-8 decode".to_string(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_common_types() {
        assert_eq!(classify_media("application/pdf", "a.pdf"), "pdf");
        assert_eq!(classify_media("image/png", "shot.png"), "image");
        assert_eq!(classify_media("audio/wav", "a.wav"), "audio");
        assert_eq!(classify_media("video/mp4", "v.mp4"), "video");
    }

    #[test]
    fn utf8_passthrough_works() {
        let result = extract_or_utf8(b"hello\r\nworld", Some("text/plain"), Some("note.txt"));
        assert!(result.success);
        assert_eq!(result.text, "hello\nworld");
    }
}
