// audio/diarization.rs
//
// Basic energy-based speaker diarization for meeting transcription.
// Segments audio by silence, extracts features, and clusters into speakers.

use log::{debug, info};
use serde::{Deserialize, Serialize};

/// Speaker segment with assigned speaker ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerSegment {
    pub speaker_id: u32,
    pub start_time: f64,
    pub end_time: f64,
    pub confidence: f32,
}

/// Audio features for speaker clustering
#[derive(Debug, Clone)]
struct AudioFeatures {
    rms_energy: f32,
    zero_crossing_rate: f32,
    spectral_centroid: f32,
}

/// Speaker diarization engine
pub struct SpeakerDiarizer {
    /// Minimum silence duration to split segments (seconds)
    silence_threshold: f64,
    /// Energy threshold for silence detection (RMS)
    silence_energy_threshold: f32,
    /// Number of speakers to cluster (auto-detected if None)
    num_speakers: Option<u32>,
    /// Maximum speakers to consider
    max_speakers: u32,
    /// Minimum speakers to consider
    min_speakers: u32,
}

impl SpeakerDiarizer {
    pub fn new() -> Self {
        Self {
            silence_threshold: 0.5, // 500ms
            silence_energy_threshold: 0.01,
            num_speakers: None,
            max_speakers: 4,
            min_speakers: 2,
        }
    }

    pub fn with_num_speakers(mut self, num: u32) -> Self {
        self.num_speakers = Some(num);
        self
    }

    /// Diarize audio samples and return speaker segments
    /// 
    /// # Arguments
    /// * `samples` - Audio samples at 16kHz mono
    /// * `sample_rate` - Sample rate (should be 16000)
    /// * `start_time` - Start time offset in seconds
    /// 
    /// # Returns
    /// Vector of speaker segments with speaker IDs
    pub fn diarize(&self, samples: &[f32], sample_rate: u32, start_time: f64) -> Vec<SpeakerSegment> {
        if samples.is_empty() {
            return Vec::new();
        }

        info!("Diarizing {} samples at {}Hz", samples.len(), sample_rate);

        // Step 1: Segment by silence
        let segments = self.segment_by_silence(samples, sample_rate);
        debug!("Found {} speech segments after silence detection", segments.len());

        if segments.is_empty() {
            return Vec::new();
        }

        // Step 2: Extract features from each segment
        let features: Vec<AudioFeatures> = segments
            .iter()
            .map(|(start, end)| self.extract_features(&samples[*start..*end]))
            .collect();

        // Step 3: Cluster segments into speakers
        let num_clusters = self.num_speakers.unwrap_or_else(|| {
            self.auto_detect_speakers(&features)
        });

        debug!("Clustering into {} speakers", num_clusters);
        let assignments = self.kmeans_cluster(&features, num_clusters);

        // Step 4: Build speaker segments
        let mut speaker_segments = Vec::new();
        for (i, ((start, end), speaker_id)) in segments.iter().zip(assignments.iter()).enumerate() {
            let segment_start = start_time + (*start as f64 / sample_rate as f64);
            let segment_end = start_time + (*end as f64 / sample_rate as f64);
            
            speaker_segments.push(SpeakerSegment {
                speaker_id: *speaker_id,
                start_time: segment_start,
                end_time: segment_end,
                confidence: 0.8, // Fixed confidence for now
            });

            debug!(
                "Segment {}: speaker {} ({:.2}s - {:.2}s)",
                i, speaker_id, segment_start, segment_end
            );
        }

        info!("Diarization complete: {} segments, {} speakers", 
              speaker_segments.len(), num_clusters);

        speaker_segments
    }

    /// Segment audio by silence gaps
    /// Returns vector of (start_sample, end_sample) pairs for speech segments
    fn segment_by_silence(&self, samples: &[f32], sample_rate: u32) -> Vec<(usize, usize)> {
        let window_size = (sample_rate as f32 * 0.03) as usize; // 30ms windows
        let silence_samples = (sample_rate as f64 * self.silence_threshold) as usize;
        
        let mut segments = Vec::new();
        let mut segment_start = 0;
        let mut silence_start = None;
        let mut in_speech = false;

        for window_start in (0..samples.len()).step_by(window_size) {
            let window_end = (window_start + window_size).min(samples.len());
            let window = &samples[window_start..window_end];
            
            // Calculate RMS energy for this window
            let rms = self.calculate_rms(window);
            let is_silent = rms < self.silence_energy_threshold;

            if is_silent {
                if silence_start.is_none() {
                    silence_start = Some(window_start);
                }
                
                // Check if silence is long enough to split
                if let Some(silence_start_sample) = silence_start {
                    let silence_duration = window_start - silence_start_sample;
                    if silence_duration >= silence_samples && in_speech {
                        // End current segment
                        segments.push((segment_start, silence_start_sample));
                        in_speech = false;
                    }
                }
            } else {
                if !in_speech {
                    // Start new speech segment
                    segment_start = window_start;
                    in_speech = true;
                }
                silence_start = None;
            }
        }

        // Close final segment if still in speech
        if in_speech {
            segments.push((segment_start, samples.len()));
        }

        segments
    }

    /// Extract audio features from a segment
    fn extract_features(&self, samples: &[f32]) -> AudioFeatures {
        let rms = self.calculate_rms(samples);
        let zcr = self.calculate_zero_crossing_rate(samples);
        let centroid = self.calculate_spectral_centroid(samples);

        AudioFeatures {
            rms_energy: rms,
            zero_crossing_rate: zcr,
            spectral_centroid: centroid,
        }
    }

    /// Calculate RMS energy
    fn calculate_rms(&self, samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum_squares: f32 = samples.iter().map(|&x| x * x).sum();
        (sum_squares / samples.len() as f32).sqrt()
    }

    /// Calculate zero-crossing rate
    fn calculate_zero_crossing_rate(&self, samples: &[f32]) -> f32 {
        if samples.len() < 2 {
            return 0.0;
        }
        
        let mut crossings = 0;
        for i in 1..samples.len() {
            if (samples[i] >= 0.0 && samples[i - 1] < 0.0) ||
               (samples[i] < 0.0 && samples[i - 1] >= 0.0) {
                crossings += 1;
            }
        }
        
        crossings as f32 / (samples.len() - 1) as f32
    }

    /// Calculate spectral centroid (simplified using FFT)
    fn calculate_spectral_centroid(&self, samples: &[f32]) -> f32 {
        // Simplified: use frequency-weighted average of absolute values
        // This is a rough approximation without full FFT
        if samples.is_empty() {
            return 0.0;
        }

        let mut weighted_sum = 0.0f32;
        let mut total_weight = 0.0f32;

        for (i, &sample) in samples.iter().enumerate() {
            let freq_weight = i as f32;
            weighted_sum += freq_weight * sample.abs();
            total_weight += sample.abs();
        }

        if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        }
    }

    /// Auto-detect number of speakers using elbow method
    fn auto_detect_speakers(&self, features: &[AudioFeatures]) -> u32 {
        if features.len() < 2 {
            return 1;
        }

        let mut best_k = self.min_speakers;
        let mut best_score = f32::MAX;

        for k in self.min_speakers..=self.max_speakers.min(features.len() as u32) {
            let assignments = self.kmeans_cluster(features, k);
            let score = self.calculate_silhouette_score(features, &assignments, k);
            
            debug!("k={}: silhouette score = {:.4}", k, score);
            
            if score < best_score {
                best_score = score;
                best_k = k;
            }
        }

        info!("Auto-detected {} speakers", best_k);
        best_k
    }

    /// K-means clustering on audio features
    fn kmeans_cluster(&self, features: &[AudioFeatures], k: u32) -> Vec<u32> {
        if features.is_empty() || k == 0 {
            return Vec::new();
        }

        let k = k as usize;
        let max_iterations = 100;
        let tolerance = 1e-4;

        // Initialize centroids (use first k points)
        let mut centroids: Vec<[f32; 3]> = features
            .iter()
            .take(k)
            .map(|f| [f.rms_energy, f.zero_crossing_rate, f.spectral_centroid])
            .collect();

        let mut assignments = vec![0u32; features.len()];

        for iteration in 0..max_iterations {
            // Assign points to nearest centroid
            let mut changed = false;
            for (i, feature) in features.iter().enumerate() {
                let point = [feature.rms_energy, feature.zero_crossing_rate, feature.spectral_centroid];
                let nearest = self.find_nearest_centroid(&point, &centroids);
                
                if assignments[i] != nearest as u32 {
                    assignments[i] = nearest as u32;
                    changed = true;
                }
            }

            // Check convergence
            if !changed && iteration > 0 {
                debug!("K-means converged after {} iterations", iteration);
                break;
            }

            // Update centroids
            let mut new_centroids = vec![[0.0f32; 3]; k];
            let mut counts = vec![0usize; k];

            for (i, feature) in features.iter().enumerate() {
                let cluster = assignments[i] as usize;
                new_centroids[cluster][0] += feature.rms_energy;
                new_centroids[cluster][1] += feature.zero_crossing_rate;
                new_centroids[cluster][2] += feature.spectral_centroid;
                counts[cluster] += 1;
            }

            for (i, count) in counts.iter().enumerate() {
                if *count > 0 {
                    new_centroids[i][0] /= *count as f32;
                    new_centroids[i][1] /= *count as f32;
                    new_centroids[i][2] /= *count as f32;
                }
            }

            // Check if centroids moved significantly
            let mut max_shift = 0.0f32;
            for (old, new) in centroids.iter().zip(new_centroids.iter()) {
                let shift = ((old[0] - new[0]).powi(2) + 
                            (old[1] - new[1]).powi(2) + 
                            (old[2] - new[2]).powi(2)).sqrt();
                max_shift = max_shift.max(shift);
            }

            centroids = new_centroids;

            if max_shift < tolerance {
                debug!("K-means converged after {} iterations (shift: {:.6})", iteration, max_shift);
                break;
            }
        }

        assignments
    }

    /// Find nearest centroid using Euclidean distance
    fn find_nearest_centroid(&self, point: &[f32; 3], centroids: &[[f32; 3]]) -> usize {
        centroids
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let dist = ((point[0] - c[0]).powi(2) + 
                           (point[1] - c[1]).powi(2) + 
                           (point[2] - c[2]).powi(2)).sqrt();
                (i, dist)
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Calculate silhouette score (lower is better for our purposes)
    fn calculate_silhouette_score(
        &self,
        features: &[AudioFeatures],
        assignments: &[u32],
        k: u32,
    ) -> f32 {
        if features.len() < 2 {
            return f32::MAX;
        }

        let mut total_score = 0.0f32;
        let mut count = 0;

        for (i, feature) in features.iter().enumerate() {
            let cluster = assignments[i];
            
            // Calculate average distance to same cluster (a)
            let mut a_sum = 0.0f32;
            let mut a_count = 0;
            for (j, other) in features.iter().enumerate() {
                if i != j && assignments[j] == cluster {
                    a_sum += self.feature_distance(feature, other);
                    a_count += 1;
                }
            }
            let a = if a_count > 0 { a_sum / a_count as f32 } else { 0.0 };

            // Calculate minimum average distance to other clusters (b)
            let mut min_b = f32::MAX;
            for other_cluster in 0..k {
                if other_cluster == cluster {
                    continue;
                }
                
                let mut b_sum = 0.0f32;
                let mut b_count = 0;
                for (j, other) in features.iter().enumerate() {
                    if assignments[j] == other_cluster {
                        b_sum += self.feature_distance(feature, other);
                        b_count += 1;
                    }
                }
                
                if b_count > 0 {
                    let b = b_sum / b_count as f32;
                    min_b = min_b.min(b);
                }
            }

            if min_b == f32::MAX {
                min_b = 0.0;
            }

            // Silhouette coefficient: (b - a) / max(a, b)
            let s = if a.max(min_b) > 0.0 {
                (min_b - a) / a.max(min_b)
            } else {
                0.0
            };

            total_score += s;
            count += 1;
        }

        // Return negative average (we want to minimize)
        if count > 0 {
            -(total_score / count as f32)
        } else {
            f32::MAX
        }
    }

    /// Calculate distance between two feature vectors
    fn feature_distance(&self, a: &AudioFeatures, b: &AudioFeatures) -> f32 {
        ((a.rms_energy - b.rms_energy).powi(2) +
         (a.zero_crossing_rate - b.zero_crossing_rate).powi(2) +
         (a.spectral_centroid - b.spectral_centroid).powi(2)).sqrt()
    }
}

impl Default for SpeakerDiarizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diarizer_creation() {
        let diarizer = SpeakerDiarizer::new();
        assert_eq!(diarizer.min_speakers, 2);
        assert_eq!(diarizer.max_speakers, 4);
    }

    #[test]
    fn test_rms_calculation() {
        let diarizer = SpeakerDiarizer::new();
        let samples = vec![0.5, -0.5, 0.5, -0.5];
        let rms = diarizer.calculate_rms(&samples);
        assert!((rms - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_zero_crossing_rate() {
        let diarizer = SpeakerDiarizer::new();
        let samples = vec![1.0, -1.0, 1.0, -1.0];
        let zcr = diarizer.calculate_zero_crossing_rate(&samples);
        assert!((zcr - 1.0).abs() < 0.01); // 3 crossings / 3 intervals
    }
}
