//! Recall benchmarks on standard ANN benchmark datasets
//!
//! Datasets:
//! - SIFT-1M: 1M 128-dim vectors from Texmex (ground truth available)
//! - GloVe-100: 1.2M 100-dim word embeddings (ground truth generated)
//!
//! # Dataset Format
//!
//! Both fvecs (float vectors) and ivecs (integer vectors) use the same format:
//! - 4 bytes: dimension (little-endian u32)
//! - dim * 4 bytes: vector data (f32 or u32, little-endian)
//!
//! # Usage
//!
//! ```ignore
//! use recall::{load_fvecs, load_ivecs, calculate_recall};
//!
//! let base = load_fvecs(Path::new("sift_base.fvecs"))?;
//! let queries = load_fvecs(Path::new("sift_query.fvecs"))?;
//! let ground_truth = load_ivecs(Path::new("sift_groundtruth.ivecs"))?;
//!
//! // Run search and calculate recall
//! let recall = calculate_recall(&search_results, &ground_truth[0], 10);
//! ```

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Error, ErrorKind, Read, Result};
use std::path::Path;

/// Load vectors in fvecs format with validation
///
/// The fvecs format stores vectors as:
/// - 4 bytes: dimension (u32, little-endian)
/// - dim * 4 bytes: vector components (f32, little-endian)
///
/// # Errors
///
/// Returns error if:
/// - File cannot be opened
/// - File is malformed (unexpected EOF, invalid dimension)
/// - Dimensions are inconsistent across vectors
/// - Dimension is 0 or > 10,000 (sanity check)
///
/// # Example
///
/// ```ignore
/// let vectors = load_fvecs(Path::new("sift_base.fvecs"))?;
/// assert_eq!(vectors[0].len(), 128); // SIFT is 128-dim
/// ```
pub fn load_fvecs(path: &Path) -> Result<Vec<Vec<f32>>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut vectors = Vec::new();
    let mut expected_dim: Option<usize> = None;

    loop {
        // Read dimension (4 bytes, little-endian u32)
        let mut dim_buf = [0u8; 4];
        match reader.read_exact(&mut dim_buf) {
            Ok(()) => {}
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        }
        let dim = u32::from_le_bytes(dim_buf) as usize;

        // Validate dimension consistency
        match expected_dim {
            None => expected_dim = Some(dim),
            Some(expected) if dim != expected => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "Dimension mismatch at vector {}: expected {}, got {}",
                        vectors.len(),
                        expected,
                        dim
                    ),
                ));
            }
            _ => {}
        }

        // Validate dimension is reasonable (sanity check)
        if dim == 0 || dim > 10_000 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Invalid dimension {} at vector {}", dim, vectors.len()),
            ));
        }

        // Read vector data
        let mut vec_buf = vec![0u8; dim * 4];
        reader.read_exact(&mut vec_buf)?;

        let vec: Vec<f32> = vec_buf
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();

        vectors.push(vec);
    }

    Ok(vectors)
}

/// Load ground truth in ivecs format with validation
///
/// The ivecs format stores integer vectors (neighbor IDs) as:
/// - 4 bytes: k (number of neighbors, u32, little-endian)
/// - k * 4 bytes: neighbor IDs (u32, little-endian)
///
/// # Errors
///
/// Returns error if:
/// - File cannot be opened
/// - File is malformed
/// - k is 0 or > 1000 (sanity check)
///
/// # Example
///
/// ```ignore
/// let gt = load_ivecs(Path::new("sift_groundtruth.ivecs"))?;
/// assert_eq!(gt[0].len(), 100); // Usually 100 ground truth neighbors
/// ```
pub fn load_ivecs(path: &Path) -> Result<Vec<Vec<u32>>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut results = Vec::new();

    loop {
        let mut k_buf = [0u8; 4];
        match reader.read_exact(&mut k_buf) {
            Ok(()) => {}
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        }
        let k = u32::from_le_bytes(k_buf) as usize;

        // Validate k is reasonable
        if k == 0 || k > 1000 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Invalid k {} at result {}", k, results.len()),
            ));
        }

        let mut ids_buf = vec![0u8; k * 4];
        reader.read_exact(&mut ids_buf)?;

        let ids: Vec<u32> = ids_buf
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();

        results.push(ids);
    }

    Ok(results)
}

/// Calculate recall@k
///
/// Recall is the fraction of ground truth neighbors found in the top-k results.
///
/// # Arguments
///
/// * `results` - Search results (vector IDs returned by the index)
/// * `ground_truth` - True nearest neighbors (from ground truth file)
/// * `k` - Number of neighbors to consider
///
/// # Returns
///
/// Recall value in range [0.0, 1.0]
///
/// # Example
///
/// ```ignore
/// let recall = calculate_recall(&[0, 1, 2, 3, 4], &[0, 1, 5, 6, 7], 5);
/// assert!((recall - 0.4).abs() < 0.001); // 2 out of 5 match
/// ```
#[must_use]
pub fn calculate_recall(results: &[u64], ground_truth: &[u32], k: usize) -> f64 {
    let k = k.min(results.len()).min(ground_truth.len());

    if k == 0 {
        return 0.0;
    }

    let result_set: HashSet<u64> = results.iter().take(k).copied().collect();
    let truth_set: HashSet<u64> = ground_truth.iter().take(k).map(|&x| u64::from(x)).collect();

    let intersection = result_set.intersection(&truth_set).count();
    intersection as f64 / k as f64
}

/// Recall benchmark results for a single configuration
#[derive(Debug, Clone)]
pub struct RecallBenchResult {
    /// Dataset name
    pub dataset: String,
    /// Index mode ("float32" or "sq8")
    pub mode: String,
    /// Number of neighbors requested
    pub k: usize,
    /// ef_search parameter value
    pub ef_search: usize,
    /// Average recall across all queries
    pub recall: f64,
    /// Queries per second
    pub queries_per_second: f64,
    /// Median latency in microseconds
    pub latency_p50_us: f64,
    /// 99th percentile latency in microseconds
    pub latency_p99_us: f64,
}

impl RecallBenchResult {
    /// Format result as a markdown table row
    #[must_use]
    pub fn as_table_row(&self) -> String {
        format!(
            "| {} | {} | {} | {} | {:.4} | {:.0} | {:.0} | {:.0} |",
            self.dataset,
            self.mode,
            self.ef_search,
            self.k,
            self.recall,
            self.queries_per_second,
            self.latency_p50_us,
            self.latency_p99_us
        )
    }
}

/// Calculate percentile from sorted values
///
/// # Panics
///
/// Panics if values is empty
#[must_use]
pub fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    assert!(
        !sorted_values.is_empty(),
        "Cannot calculate percentile of empty slice"
    );
    let idx = ((sorted_values.len() as f64 * p) as usize).min(sorted_values.len() - 1);
    sorted_values[idx]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Create a test fvecs file
    fn create_test_fvecs(vectors: &[Vec<f32>]) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        for vec in vectors {
            let dim = vec.len() as u32;
            file.write_all(&dim.to_le_bytes()).unwrap();
            for &val in vec {
                file.write_all(&val.to_le_bytes()).unwrap();
            }
        }
        file.flush().unwrap();
        file
    }

    /// Create a test ivecs file
    fn create_test_ivecs(results: &[Vec<u32>]) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        for ids in results {
            let k = ids.len() as u32;
            file.write_all(&k.to_le_bytes()).unwrap();
            for &id in ids {
                file.write_all(&id.to_le_bytes()).unwrap();
            }
        }
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_load_fvecs_valid() {
        let vectors = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];
        let file = create_test_fvecs(&vectors);

        let loaded = load_fvecs(file.path()).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0], vec![1.0, 2.0, 3.0]);
        assert_eq!(loaded[1], vec![4.0, 5.0, 6.0]);
    }

    #[test]
    fn test_load_fvecs_empty() {
        let file = NamedTempFile::new().unwrap();
        let loaded = load_fvecs(file.path()).unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_load_fvecs_inconsistent_dimensions() {
        let mut file = NamedTempFile::new().unwrap();
        // First vector: dim=3
        file.write_all(&3u32.to_le_bytes()).unwrap();
        file.write_all(&1.0f32.to_le_bytes()).unwrap();
        file.write_all(&2.0f32.to_le_bytes()).unwrap();
        file.write_all(&3.0f32.to_le_bytes()).unwrap();
        // Second vector: dim=4 (inconsistent!)
        file.write_all(&4u32.to_le_bytes()).unwrap();
        file.write_all(&1.0f32.to_le_bytes()).unwrap();
        file.write_all(&2.0f32.to_le_bytes()).unwrap();
        file.write_all(&3.0f32.to_le_bytes()).unwrap();
        file.write_all(&4.0f32.to_le_bytes()).unwrap();
        file.flush().unwrap();

        let result = load_fvecs(file.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Dimension mismatch"));
    }

    #[test]
    fn test_load_fvecs_invalid_dimension_zero() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&0u32.to_le_bytes()).unwrap();
        file.flush().unwrap();

        let result = load_fvecs(file.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid dimension"));
    }

    #[test]
    fn test_load_fvecs_invalid_dimension_too_large() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&20000u32.to_le_bytes()).unwrap();
        file.flush().unwrap();

        let result = load_fvecs(file.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid dimension"));
    }

    #[test]
    fn test_load_ivecs_valid() {
        let results = vec![vec![0, 1, 2], vec![3, 4, 5]];
        let file = create_test_ivecs(&results);

        let loaded = load_ivecs(file.path()).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0], vec![0, 1, 2]);
        assert_eq!(loaded[1], vec![3, 4, 5]);
    }

    #[test]
    fn test_load_ivecs_empty() {
        let file = NamedTempFile::new().unwrap();
        let loaded = load_ivecs(file.path()).unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_load_ivecs_invalid_k_zero() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&0u32.to_le_bytes()).unwrap();
        file.flush().unwrap();

        let result = load_ivecs(file.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid k"));
    }

    #[test]
    fn test_calculate_recall_perfect() {
        let results = vec![0u64, 1, 2, 3, 4];
        let ground_truth = vec![0u32, 1, 2, 3, 4];
        let recall = calculate_recall(&results, &ground_truth, 5);
        assert!((recall - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_recall_zero() {
        let results = vec![10u64, 11, 12, 13, 14];
        let ground_truth = vec![0u32, 1, 2, 3, 4];
        let recall = calculate_recall(&results, &ground_truth, 5);
        assert!((recall - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_recall_partial() {
        let results = vec![0u64, 1, 10, 11, 12];
        let ground_truth = vec![0u32, 1, 2, 3, 4];
        let recall = calculate_recall(&results, &ground_truth, 5);
        assert!((recall - 0.4).abs() < 0.001); // 2 out of 5
    }

    #[test]
    fn test_calculate_recall_k_larger_than_results() {
        let results = vec![0u64, 1];
        let ground_truth = vec![0u32, 1, 2, 3, 4];
        let recall = calculate_recall(&results, &ground_truth, 5);
        assert!((recall - 1.0).abs() < 0.001); // Both results match
    }

    #[test]
    fn test_calculate_recall_empty() {
        let results: Vec<u64> = vec![];
        let ground_truth = vec![0u32, 1, 2];
        let recall = calculate_recall(&results, &ground_truth, 5);
        assert!((recall - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_percentile_median() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let p50 = percentile(&values, 0.5);
        assert!((p50 - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_percentile_p99() {
        let values: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let p99 = percentile(&values, 0.99);
        assert!((p99 - 99.0).abs() < 0.001);
    }

    #[test]
    fn test_recall_result_table_row() {
        let result = RecallBenchResult {
            dataset: "SIFT-1M".to_string(),
            mode: "float32".to_string(),
            k: 10,
            ef_search: 50,
            recall: 0.9512,
            queries_per_second: 5000.0,
            latency_p50_us: 150.0,
            latency_p99_us: 500.0,
        };
        let row = result.as_table_row();
        assert!(row.contains("SIFT-1M"));
        assert!(row.contains("float32"));
        assert!(row.contains("0.9512"));
    }
}
