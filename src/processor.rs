use anyhow::Result;
use std::path::{Path, PathBuf};
use crate::config::Config;
use crate::ffmpeg::{self, SilenceSegment};
use tracing::info;

pub struct Processor {
    config: Config,
}

impl Processor {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn run(&self, input: &Path, output_dir: &Path) -> Result<PathBuf> {
        info!("Analyzing video: {:?}", input);
        let total_duration = ffmpeg::get_duration(input).await?;
        
        info!("Detecting silence segments...");
        let silences = ffmpeg::detect_silence(input, &self.config).await?;
        
        let keep_segments = self.calculate_keep_segments(&silences, total_duration);
        info!("Found {} highlight segments", keep_segments.len());

        if keep_segments.is_empty() {
            return Err(anyhow::anyhow!("No highlights detected with current settings."));
        }

        let mut segment_paths = Vec::new();
        tokio::fs::create_dir_all(output_dir).await?;

        for (i, (start, duration)) in keep_segments.iter().enumerate() {
            let segment_name = format!("segment_{:03}.mp4", i);
            let segment_path = output_dir.join(segment_name);
            
            info!("Extracting segment {}/{}: start={:.2}s, duration={:.2}s", i + 1, keep_segments.len(), start, duration);
            ffmpeg::extract_segment(input, &segment_path, *start, *duration).await?;
            segment_paths.push(segment_path);
        }

        let output_filename = format!(
            "{}_highlights.mp4",
            input.file_stem().unwrap().to_str().unwrap()
        );
        let final_output = output_dir.join(output_filename);

        info!("Concatenating segments into final video...");
        let concat_file = output_dir.join("concat_list.txt");
        let mut list_content = String::new();
        for path in &segment_paths {
            let abs_path = std::fs::canonicalize(path)?;
            list_content.push_str(&format!("file '{}'\n", abs_path.to_str().unwrap()));
        }
        tokio::fs::write(&concat_file, list_content).await?;

        ffmpeg::concatenate_segments(&concat_file, &final_output).await?;

        // Cleanup segments
        for path in segment_paths {
            let _ = tokio::fs::remove_file(path).await;
        }
        let _ = tokio::fs::remove_file(concat_file).await;

        info!("Processing complete: {:?}", final_output);
        Ok(final_output)
    }

    pub fn calculate_keep_segments(&self, silences: &[SilenceSegment], total_duration: f64) -> Vec<(f64, f64)> {
        let mut raw_segments = Vec::new();
        let mut last_end = 0.0;

        for silence in silences {
            let seg_start = last_end;
            let seg_end = (silence.start - self.config.margin_s).max(seg_start);
            let duration = seg_end - seg_start;

            if duration >= 0.1 {
                raw_segments.push((seg_start, duration));
            }
            last_end = silence.end + self.config.margin_s;
        }

        if last_end < total_duration {
            let duration = total_duration - last_end;
            if duration >= 0.1 {
                raw_segments.push((last_end, duration));
            }
        }

        let coalesced = self.coalesce_segments(raw_segments);
        let mut final_segments = Vec::new();
        
        for (start, duration) in coalesced {
            self.add_segments(&mut final_segments, start, duration);
        }

        final_segments
    }

    pub fn coalesce_segments(&self, raw_segments: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
        if raw_segments.is_empty() {
            return Vec::new();
        }

        let mut coalesced = Vec::new();
        let mut current = raw_segments[0];

        for &next in raw_segments.iter().skip(1) {
            let current_end = current.0 + current.1;
            let gap = next.0 - current_end;

            // If the gap is smaller than twice the margin, merge them
            if gap < self.config.margin_s * 2.1 {
                current.1 = (next.0 + next.1) - current.0;
            } else {
                if current.1 >= self.config.min_clip_length {
                    coalesced.push(current);
                }
                current = next;
            }
        }

        if current.1 >= self.config.min_clip_length {
            coalesced.push(current);
        }

        coalesced
    }

    fn add_segments(&self, segments: &mut Vec<(f64, f64)>, start: f64, duration: f64) {
        if duration <= self.config.max_clip_length {
            segments.push((start, duration));
        } else {
            // Split long segments
            let mut current = start;
            let mut remaining = duration;
            while remaining > 0.0 {
                let chunk_dur = remaining.min(self.config.max_clip_length);
                if chunk_dur >= self.config.min_clip_length {
                    segments.push((current, chunk_dur));
                }
                current += chunk_dur;
                remaining -= chunk_dur;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_calculate_keep_segments() {
        let config = Config {
            min_clip_length: 2.0,
            max_clip_length: 10.0,
            margin_s: 1.0,
            ..Default::default()
        };
        let processor = Processor::new(config);
        
        let silences = vec![
            SilenceSegment { start: 5.0, end: 10.0 },
            SilenceSegment { start: 20.0, end: 25.0 },
        ];
        let total_duration = 40.0;
        
        // Segments:
        // 0.0 to 5.0 - 1.0 = 4.0 (Keep)
        // 10.0 + 1.0 = 11.0 to 20.0 - 1.0 = 19.0 (Duration 8.0, Keep)
        // 25.0 + 1.0 = 26.0 to 40.0 (Duration 14.0, Split into 10.0 and 4.0)
        
        let segments = processor.calculate_keep_segments(&silences, total_duration);
        assert_eq!(segments.len(), 4);
        assert_eq!(segments[0], (0.0, 4.0));
        assert_eq!(segments[1], (11.0, 8.0));
        assert_eq!(segments[2], (26.0, 10.0));
        assert_eq!(segments[3], (36.0, 4.0));
    }

    #[test]
    fn test_add_segments_split() {
        let config = Config {
            min_clip_length: 1.0,
            max_clip_length: 5.0,
            ..Default::default()
        };
        let processor = Processor::new(config);
        let mut segments = Vec::new();
        
        processor.add_segments(&mut segments, 0.0, 12.0);
        // Expected: 5.0, 5.0, 2.0
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0], (0.0, 5.0));
        assert_eq!(segments[1], (5.0, 5.0));
        assert_eq!(segments[2], (10.0, 2.0));
    }
}
