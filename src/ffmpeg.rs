use anyhow::{Result, anyhow};
use std::path::Path;
use tokio::process::Command;
use crate::config::Config;

pub struct SilenceSegment {
    pub start: f64,
    pub end: f64,
}

pub async fn detect_silence(input: &Path, config: &Config) -> Result<Vec<SilenceSegment>> {
    let output = Command::new("ffmpeg")
        .args([
            "-i",
            input.to_str().ok_or_else(|| anyhow!("Invalid input path"))?,
            "-af",
            &format!(
                "silencedetect=noise={}dB:d={}",
                config.silence_threshold_db, config.min_silence_duration
            ),
            "-f",
            "null",
            "-",
        ])
        .output()
        .await?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    Ok(parse_silence_output(&stderr))
}

fn parse_silence_output(output: &str) -> Vec<SilenceSegment> {
    let mut silences = Vec::new();
    let mut current_start = None;

    for line in output.lines() {
        if line.contains("silence_start") {
            if let Some(start) = extract_timestamp(line) {
                current_start = Some(start);
            }
        } else if line.contains("silence_end") {
            if let (Some(start), Some(end)) = (current_start, extract_timestamp(line)) {
                silences.push(SilenceSegment { start, end });
                current_start = None;
            }
        }
    }
    silences
}

fn extract_timestamp(line: &str) -> Option<f64> {
    line.split(':')
        .last()?
        .trim()
        .split('|')
        .next()?
        .trim()
        .parse::<f64>()
        .ok()
}

pub async fn extract_segment(
    input: &Path,
    output: &Path,
    start: f64,
    duration: f64,
) -> Result<()> {
    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-ss",
            &start.to_string(),
            "-i",
            input.to_str().ok_or_else(|| anyhow!("Invalid input path"))?,
            "-t",
            &duration.to_string(),
            "-c:v",
            "h264_videotoolbox", // Hardware accelerated on Apple Silicon
            "-b:v",
            "6000k",             // Solid bitrate for 1024-1080p
            "-c:a",
            "aac",
            "-af",
            &format!(
                "afade=t=in:d=0.05,afade=t=out:st={:.3}:d=0.05",
                (duration - 0.05).max(0.0)
            ),
            output.to_str().ok_or_else(|| anyhow!("Invalid output path"))?,
        ])
        .status()
        .await?;

    if !status.success() {
        return Err(anyhow!("FFmpeg failed to extract segment"));
    }
    Ok(())
}

pub async fn concatenate_segments(concat_file: &Path, output: &Path) -> Result<()> {
    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            concat_file.to_str().ok_or_else(|| anyhow!("Invalid concat file path"))?,
            "-fflags",
            "+genpts", // Regenerate timestamps to avoid Non-monotonic DTS
            "-c",
            "copy",
            output.to_str().ok_or_else(|| anyhow!("Invalid output path"))?,
        ])
        .status()
        .await?;

    if !status.success() {
        return Err(anyhow!("FFmpeg failed to concatenate segments"));
    }
    Ok(())
}

pub async fn get_duration(input: &Path) -> Result<f64> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            input.to_str().ok_or_else(|| anyhow!("Invalid input path"))?,
        ])
        .output()
        .await?;

    let dur_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    dur_str.parse::<f64>().map_err(|e| anyhow!("Failed to parse duration: {}", e))
}
