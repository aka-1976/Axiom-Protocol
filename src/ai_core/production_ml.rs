// src/ai_core/production_ml.rs
// Production-grade ML anomaly detection stack for Axiom Protocol

use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::BinaryHeap;
use std::f64::consts::PI;

// ---------------------------------------------------------------------------
// Utility
// ---------------------------------------------------------------------------

pub fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}

fn box_muller(rng: &mut impl Rng) -> f64 {
    let u1: f64 = rng.gen_range(1e-10_f64..1.0_f64);
    let u2: f64 = rng.gen_range(0.0_f64..1.0_f64);
    (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos()
}

// ---------------------------------------------------------------------------
// KdTree
// ---------------------------------------------------------------------------

struct KdNode {
    point: Vec<f64>,
    point_index: usize,
    split_dim: usize,
    left: Option<Box<KdNode>>,
    right: Option<Box<KdNode>>,
}

pub struct KdTree {
    root: Option<Box<KdNode>>,
    dimension: usize,
}

#[derive(Clone)]
struct Neighbor {
    index: usize,
    distance: f64,
}

impl PartialEq for Neighbor {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl Eq for Neighbor {}

impl PartialOrd for Neighbor {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Neighbor {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.distance
            .partial_cmp(&other.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl KdTree {
    pub fn build(points: &[Vec<f64>]) -> Self {
        if points.is_empty() {
            return KdTree {
                root: None,
                dimension: 0,
            };
        }
        let dimension = points[0].len();
        let mut indices: Vec<usize> = (0..points.len()).collect();
        let root = Self::build_recursive(points, &mut indices, 0, dimension);
        KdTree {
            root: Some(root),
            dimension,
        }
    }

    fn build_recursive(
        points: &[Vec<f64>],
        indices: &mut [usize],
        depth: usize,
        dimension: usize,
    ) -> Box<KdNode> {
        let split_dim = depth % dimension;
        indices.sort_by(|&a, &b| {
            points[a][split_dim]
                .partial_cmp(&points[b][split_dim])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let median = indices.len() / 2;
        let point_index = indices[median];

        let left = if median > 0 {
            let mut left_indices = indices[..median].to_vec();
            Some(Self::build_recursive(
                points,
                &mut left_indices,
                depth + 1,
                dimension,
            ))
        } else {
            None
        };

        let right = if median + 1 < indices.len() {
            let mut right_indices = indices[median + 1..].to_vec();
            Some(Self::build_recursive(
                points,
                &mut right_indices,
                depth + 1,
                dimension,
            ))
        } else {
            None
        };

        Box::new(KdNode {
            point: points[point_index].clone(),
            point_index,
            split_dim,
            left,
            right,
        })
    }

    pub fn knn(&self, query: &[f64], k: usize) -> Vec<(usize, f64)> {
        if self.root.is_none() || k == 0 {
            return Vec::new();
        }
        let mut heap: BinaryHeap<Neighbor> = BinaryHeap::new();
        self.knn_recursive(self.root.as_deref().unwrap(), query, k, &mut heap);
        let mut results: Vec<(usize, f64)> =
            heap.into_iter().map(|n| (n.index, n.distance)).collect();
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    fn knn_recursive(
        &self,
        node: &KdNode,
        query: &[f64],
        k: usize,
        heap: &mut BinaryHeap<Neighbor>,
    ) {
        let dist = euclidean_distance(&node.point, query);
        if heap.len() < k {
            heap.push(Neighbor {
                index: node.point_index,
                distance: dist,
            });
        } else if let Some(top) = heap.peek() {
            if dist < top.distance {
                heap.pop();
                heap.push(Neighbor {
                    index: node.point_index,
                    distance: dist,
                });
            }
        }

        let diff = query[node.split_dim] - node.point[node.split_dim];
        let (first, second) = if diff < 0.0 {
            (&node.left, &node.right)
        } else {
            (&node.right, &node.left)
        };

        if let Some(ref child) = first {
            self.knn_recursive(child, query, k, heap);
        }

        let worst = heap
            .peek()
            .map(|n| n.distance)
            .unwrap_or(f64::INFINITY);
        if heap.len() < k || diff.abs() < worst {
            if let Some(ref child) = second {
                self.knn_recursive(child, query, k, heap);
            }
        }
    }

    pub fn range_query(&self, query: &[f64], radius: f64) -> Vec<(usize, f64)> {
        let mut results = Vec::new();
        if let Some(ref root) = self.root {
            self.range_recursive(root, query, radius, &mut results);
        }
        results
    }

    fn range_recursive(
        &self,
        node: &KdNode,
        query: &[f64],
        radius: f64,
        results: &mut Vec<(usize, f64)>,
    ) {
        let dist = euclidean_distance(&node.point, query);
        if dist <= radius {
            results.push((node.point_index, dist));
        }

        let diff = query[node.split_dim] - node.point[node.split_dim];

        if diff < 0.0 {
            if let Some(ref left) = node.left {
                self.range_recursive(left, query, radius, results);
            }
            if diff.abs() <= radius {
                if let Some(ref right) = node.right {
                    self.range_recursive(right, query, radius, results);
                }
            }
        } else {
            if let Some(ref right) = node.right {
                self.range_recursive(right, query, radius, results);
            }
            if diff.abs() <= radius {
                if let Some(ref left) = node.left {
                    self.range_recursive(left, query, radius, results);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Isolation Forest
// ---------------------------------------------------------------------------

enum IsolationNode {
    Internal {
        split_feature: usize,
        split_value: f64,
        left: Box<IsolationNode>,
        right: Box<IsolationNode>,
    },
    Leaf {
        size: usize,
    },
}

struct IsolationTree {
    root: IsolationNode,
}

impl IsolationTree {
    fn build(data: &[Vec<f64>], height_limit: usize, rng: &mut impl Rng) -> Self {
        let root = Self::build_recursive(data, 0, height_limit, rng);
        IsolationTree { root }
    }

    fn build_recursive(
        data: &[Vec<f64>],
        depth: usize,
        height_limit: usize,
        rng: &mut impl Rng,
    ) -> IsolationNode {
        if data.len() <= 1 || depth >= height_limit {
            return IsolationNode::Leaf { size: data.len() };
        }
        let dim = data[0].len();
        if dim == 0 {
            return IsolationNode::Leaf { size: data.len() };
        }
        let split_feature = rng.gen_range(0..dim);
        let mut min_val = f64::INFINITY;
        let mut max_val = f64::NEG_INFINITY;
        for point in data {
            let v = point[split_feature];
            if v < min_val {
                min_val = v;
            }
            if v > max_val {
                max_val = v;
            }
        }
        if (max_val - min_val).abs() < 1e-10 {
            return IsolationNode::Leaf { size: data.len() };
        }
        let split_value = rng.gen_range(min_val..max_val);
        let mut left_data = Vec::new();
        let mut right_data = Vec::new();
        for point in data {
            if point[split_feature] < split_value {
                left_data.push(point.clone());
            } else {
                right_data.push(point.clone());
            }
        }
        IsolationNode::Internal {
            split_feature,
            split_value,
            left: Box::new(Self::build_recursive(&left_data, depth + 1, height_limit, rng)),
            right: Box::new(Self::build_recursive(
                &right_data,
                depth + 1,
                height_limit,
                rng,
            )),
        }
    }

    fn path_length(&self, point: &[f64]) -> f64 {
        Self::path_length_recursive(&self.root, point, 0)
    }

    fn path_length_recursive(node: &IsolationNode, point: &[f64], depth: usize) -> f64 {
        match node {
            IsolationNode::Leaf { size } => {
                depth as f64 + c_factor(*size)
            }
            IsolationNode::Internal {
                split_feature,
                split_value,
                left,
                right,
            } => {
                if point[*split_feature] < *split_value {
                    Self::path_length_recursive(left, point, depth + 1)
                } else {
                    Self::path_length_recursive(right, point, depth + 1)
                }
            }
        }
    }
}

/// Average path length of unsuccessful search in BST: c(n) = 2*H(n-1) - 2*(n-1)/n
fn c_factor(n: usize) -> f64 {
    if n <= 1 {
        return 0.0;
    }
    let n_f = n as f64;
    2.0 * harmonic(n - 1) - 2.0 * (n_f - 1.0) / n_f
}

fn harmonic(n: usize) -> f64 {
    if n == 0 {
        return 0.0;
    }
    // H(n) ≈ ln(n) + Euler-Mascheroni constant
    (n as f64).ln() + 0.5772156649
}

pub struct IsolationForest {
    trees: Vec<IsolationTree>,
    subsample_size: usize,
    num_trees: usize,
    trained: bool,
}

impl IsolationForest {
    pub fn new(num_trees: usize, subsample_size: usize) -> Self {
        IsolationForest {
            trees: Vec::new(),
            subsample_size,
            num_trees,
            trained: false,
        }
    }

    pub fn fit(&mut self, data: &[Vec<f64>]) {
        if data.is_empty() {
            return;
        }
        let mut rng = rand::thread_rng();
        let height_limit = (self.subsample_size as f64).log2().ceil() as usize;
        self.trees.clear();

        for _ in 0..self.num_trees {
            let sample: Vec<Vec<f64>> = if data.len() <= self.subsample_size {
                data.to_vec()
            } else {
                let mut indices: Vec<usize> = (0..data.len()).collect();
                indices.shuffle(&mut rng);
                indices[..self.subsample_size]
                    .iter()
                    .map(|&i| data[i].clone())
                    .collect()
            };
            self.trees.push(IsolationTree::build(&sample, height_limit, &mut rng));
        }
        self.trained = true;
    }

    pub fn score(&self, point: &[f64]) -> f64 {
        if !self.trained || self.trees.is_empty() {
            return 0.5;
        }
        let avg_path: f64 = self.trees.iter().map(|t| t.path_length(point)).sum::<f64>()
            / self.trees.len() as f64;
        let c_n = c_factor(self.subsample_size);
        if c_n <= 0.0 {
            return 0.5;
        }
        2.0_f64.powf(-avg_path / c_n)
    }
}

// ---------------------------------------------------------------------------
// One-Class SVM with Random Fourier Features
// ---------------------------------------------------------------------------

pub struct OneClassSVM {
    random_weights: Vec<Vec<f64>>,
    random_offsets: Vec<f64>,
    rff_dimension: usize,
    center: Vec<f64>,
    radius: f64,
    gamma: f64,
    trained: bool,
}

impl OneClassSVM {
    pub fn new(rff_dimension: usize, gamma: f64) -> Self {
        OneClassSVM {
            random_weights: Vec::new(),
            random_offsets: Vec::new(),
            rff_dimension,
            center: Vec::new(),
            radius: 1.0,
            gamma,
            trained: false,
        }
    }

    pub fn fit(&mut self, data: &[Vec<f64>]) {
        if data.is_empty() {
            return;
        }
        let dim = data[0].len();
        let mut rng = rand::thread_rng();

        // Generate random weights ~ N(0, 1) using Box-Muller
        self.random_weights = (0..self.rff_dimension)
            .map(|_| (0..dim).map(|_| box_muller(&mut rng)).collect())
            .collect();

        // Generate random offsets ~ U(0, 2π)
        self.random_offsets = (0..self.rff_dimension)
            .map(|_| rng.gen_range(0.0..(2.0 * PI)))
            .collect();

        // Transform all data points
        let transformed: Vec<Vec<f64>> = data.iter().map(|x| self.transform(x)).collect();

        // Compute mean center
        let n = transformed.len() as f64;
        self.center = vec![0.0; self.rff_dimension];
        for t in &transformed {
            for (i, v) in t.iter().enumerate() {
                self.center[i] += v;
            }
        }
        for c in &mut self.center {
            *c /= n;
        }

        // Compute distances from center and find 95th percentile as radius
        let mut distances: Vec<f64> = transformed
            .iter()
            .map(|t| euclidean_distance(t, &self.center))
            .collect();
        distances.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx_95 = ((distances.len() as f64) * 0.95).ceil() as usize;
        let idx_95 = idx_95.min(distances.len().saturating_sub(1));
        self.radius = distances[idx_95].max(1e-10);

        self.trained = true;
    }

    pub fn transform(&self, x: &[f64]) -> Vec<f64> {
        let scale = (2.0 / self.rff_dimension as f64).sqrt();
        self.random_weights
            .iter()
            .zip(self.random_offsets.iter())
            .map(|(w, &b)| {
                let dot: f64 = w.iter().zip(x.iter()).map(|(wi, xi)| wi * xi).sum();
                scale * (self.gamma * dot + b).cos()
            })
            .collect()
    }

    pub fn score(&self, point: &[f64]) -> f64 {
        if !self.trained {
            return 0.5;
        }
        let transformed = self.transform(point);
        let dist = euclidean_distance(&transformed, &self.center);
        (dist / self.radius).min(1.0).max(0.0)
    }
}

// ---------------------------------------------------------------------------
// Local Outlier Factor (LOF)
// ---------------------------------------------------------------------------

pub struct LOFDetector {
    kd_tree: Option<KdTree>,
    data: Vec<Vec<f64>>,
    k: usize,
    k_distances: Vec<f64>,
    lrd_values: Vec<f64>,
    trained: bool,
}

impl LOFDetector {
    pub fn new(k: usize) -> Self {
        LOFDetector {
            kd_tree: None,
            data: Vec::new(),
            k,
            k_distances: Vec::new(),
            lrd_values: Vec::new(),
            trained: false,
        }
    }

    pub fn fit(&mut self, data: &[Vec<f64>]) {
        if data.is_empty() {
            return;
        }
        self.data = data.to_vec();
        self.kd_tree = Some(KdTree::build(data));

        let n = data.len();
        let k = self.k.min(n.saturating_sub(1)).max(1);

        // Precompute k-distances for each training point
        self.k_distances = vec![0.0; n];
        let mut all_neighbors: Vec<Vec<(usize, f64)>> = Vec::with_capacity(n);

        for i in 0..n {
            // k+1 because the point itself is in the tree
            let neighbors = self.kd_tree.as_ref().unwrap().knn(&data[i], k + 1);
            let filtered: Vec<(usize, f64)> =
                neighbors.into_iter().filter(|(idx, _)| *idx != i).collect();
            let k_dist = if filtered.len() >= k {
                filtered[k - 1].1
            } else if !filtered.is_empty() {
                filtered.last().unwrap().1
            } else {
                0.0
            };
            self.k_distances[i] = k_dist;
            all_neighbors.push(filtered);
        }

        // Compute Local Reachability Density for each point
        self.lrd_values = vec![0.0; n];
        for i in 0..n {
            let neighbors = &all_neighbors[i];
            let actual_k = neighbors.len().min(k);
            if actual_k == 0 {
                self.lrd_values[i] = 1.0;
                continue;
            }
            let reach_dist_sum: f64 = neighbors[..actual_k]
                .iter()
                .map(|&(j, dist)| dist.max(self.k_distances[j]))
                .sum();
            let avg_reach = reach_dist_sum / actual_k as f64;
            self.lrd_values[i] = if avg_reach > 1e-10 {
                1.0 / avg_reach
            } else {
                1.0
            };
        }

        self.trained = true;
    }

    pub fn score(&self, point: &[f64]) -> f64 {
        if !self.trained || self.kd_tree.is_none() {
            return 0.5;
        }
        let tree = self.kd_tree.as_ref().unwrap();
        let k = self.k.min(self.data.len()).max(1);
        let neighbors = tree.knn(point, k);

        if neighbors.is_empty() {
            return 0.5;
        }

        // Compute LRD of query point
        let reach_dist_sum: f64 = neighbors
            .iter()
            .map(|&(j, dist)| dist.max(self.k_distances[j]))
            .sum();
        let avg_reach = reach_dist_sum / neighbors.len() as f64;
        let lrd_query = if avg_reach > 1e-10 {
            1.0 / avg_reach
        } else {
            1.0
        };

        // LOF = average ratio of neighbor LRD to query LRD
        let lof: f64 = neighbors
            .iter()
            .map(|&(j, _)| self.lrd_values[j] / lrd_query)
            .sum::<f64>()
            / neighbors.len() as f64;

        // Normalize to [0, 1]
        ((lof - 1.0).max(0.0) / 5.0).min(1.0)
    }
}

// ---------------------------------------------------------------------------
// DBSCAN
// ---------------------------------------------------------------------------

pub struct DBSCAN {
    epsilon: f64,
    min_points: usize,
    kd_tree: Option<KdTree>,
    data: Vec<Vec<f64>>,
    labels: Vec<i32>,
    trained: bool,
}

impl DBSCAN {
    pub fn new(epsilon: f64, min_points: usize) -> Self {
        DBSCAN {
            epsilon,
            min_points,
            kd_tree: None,
            data: Vec::new(),
            labels: Vec::new(),
            trained: false,
        }
    }

    pub fn fit(&mut self, data: &[Vec<f64>]) {
        if data.is_empty() {
            return;
        }
        let n = data.len();
        self.data = data.to_vec();
        self.kd_tree = Some(KdTree::build(data));
        self.labels = vec![-1i32; n]; // -1 = unvisited / noise
        let mut visited = vec![false; n];
        let mut cluster_id: i32 = 0;

        for i in 0..n {
            if visited[i] {
                continue;
            }
            visited[i] = true;

            let neighbors = self
                .kd_tree
                .as_ref()
                .unwrap()
                .range_query(&data[i], self.epsilon);

            if neighbors.len() < self.min_points {
                // remains noise (-1)
                continue;
            }

            self.labels[i] = cluster_id;
            let mut seeds: Vec<usize> = neighbors.iter().map(|&(idx, _)| idx).collect();
            let mut seed_set: std::collections::HashSet<usize> =
                seeds.iter().copied().collect();
            let mut j = 0;
            while j < seeds.len() {
                let q = seeds[j];
                if !visited[q] {
                    visited[q] = true;
                    let q_neighbors = self
                        .kd_tree
                        .as_ref()
                        .unwrap()
                        .range_query(&data[q], self.epsilon);
                    if q_neighbors.len() >= self.min_points {
                        for &(idx, _) in &q_neighbors {
                            if seed_set.insert(idx) {
                                seeds.push(idx);
                            }
                        }
                    }
                }
                if self.labels[q] == -1 {
                    self.labels[q] = cluster_id;
                }
                j += 1;
            }
            cluster_id += 1;
        }

        self.trained = true;
    }

    pub fn score(&self, point: &[f64]) -> f64 {
        if !self.trained || self.data.is_empty() {
            return 0.5;
        }

        // Compute cluster centers
        let max_label = self.labels.iter().copied().max().unwrap_or(-1);
        if max_label < 0 {
            // All points are noise
            return 1.0;
        }

        let mut centers: Vec<Vec<f64>> = Vec::new();
        for c in 0..=max_label {
            let dim = self.data[0].len();
            let mut center = vec![0.0; dim];
            let mut count = 0usize;
            for (i, label) in self.labels.iter().enumerate() {
                if *label == c {
                    for (j, v) in self.data[i].iter().enumerate() {
                        center[j] += v;
                    }
                    count += 1;
                }
            }
            if count > 0 {
                for v in &mut center {
                    *v /= count as f64;
                }
            }
            centers.push(center);
        }

        // Distance to nearest cluster center, normalized by epsilon
        let min_dist = centers
            .iter()
            .map(|c| euclidean_distance(point, c))
            .fold(f64::INFINITY, f64::min);

        (min_dist / self.epsilon).min(1.0).max(0.0)
    }
}

// ---------------------------------------------------------------------------
// Production ML Stack (weighted ensemble)
// ---------------------------------------------------------------------------

pub struct ProductionMLStack {
    pub isolation_forest: IsolationForest,
    pub one_class_svm: OneClassSVM,
    pub lof_detector: LOFDetector,
    pub dbscan: DBSCAN,
    trained: bool,
}

impl ProductionMLStack {
    pub fn new() -> Self {
        ProductionMLStack {
            isolation_forest: IsolationForest::new(100, 256),
            one_class_svm: OneClassSVM::new(200, 0.1),
            lof_detector: LOFDetector::new(20),
            dbscan: DBSCAN::new(0.5, 5),
            trained: false,
        }
    }

    pub fn fit(&mut self, normal_data: &[Vec<f64>]) {
        self.isolation_forest.fit(normal_data);
        self.one_class_svm.fit(normal_data);
        self.lof_detector.fit(normal_data);
        self.dbscan.fit(normal_data);
        self.trained = true;
    }

    pub fn detect_anomaly(&self, features: &[f64]) -> f64 {
        if !self.trained {
            return 0.5;
        }
        let if_score = self.isolation_forest.score(features);
        let svm_score = self.one_class_svm.score(features);
        let lof_score = self.lof_detector.score(features);
        let dbscan_score = self.dbscan.score(features);

        0.35 * if_score + 0.30 * svm_score + 0.25 * lof_score + 0.10 * dbscan_score
    }

    pub fn is_trained(&self) -> bool {
        self.trained
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cluster(center: &[f64], n: usize, spread: f64) -> Vec<Vec<f64>> {
        let mut rng = rand::thread_rng();
        (0..n)
            .map(|_| {
                center
                    .iter()
                    .map(|&c| c + rng.gen_range(-spread..spread))
                    .collect()
            })
            .collect()
    }

    #[test]
    fn test_kd_tree_build_and_knn() {
        let points = vec![
            vec![1.0, 2.0],
            vec![3.0, 4.0],
            vec![5.0, 6.0],
            vec![7.0, 8.0],
            vec![2.0, 3.0],
        ];
        let tree = KdTree::build(&points);
        let neighbors = tree.knn(&[2.0, 3.0], 2);
        assert_eq!(neighbors.len(), 2);
        // Closest should be point index 4 (exact match)
        assert_eq!(neighbors[0].0, 4);
        assert!(neighbors[0].1 < 1e-10);
    }

    #[test]
    fn test_kd_tree_range_query() {
        let points = vec![
            vec![0.0, 0.0],
            vec![1.0, 0.0],
            vec![0.0, 1.0],
            vec![10.0, 10.0],
        ];
        let tree = KdTree::build(&points);
        let results = tree.range_query(&[0.0, 0.0], 1.5);
        // Should find points 0, 1, 2 but not 3
        assert_eq!(results.len(), 3);
        let indices: Vec<usize> = results.iter().map(|r| r.0).collect();
        assert!(indices.contains(&0));
        assert!(indices.contains(&1));
        assert!(indices.contains(&2));
        assert!(!indices.contains(&3));
    }

    #[test]
    fn test_isolation_forest_scoring() {
        let normal = make_cluster(&[0.0, 0.0], 200, 1.0);
        let mut forest = IsolationForest::new(100, 256);
        forest.fit(&normal);

        let normal_score = forest.score(&[0.0, 0.0]);
        let outlier_score = forest.score(&[20.0, 20.0]);
        // Outlier should have higher anomaly score
        assert!(
            outlier_score > normal_score,
            "outlier_score={} should be > normal_score={}",
            outlier_score,
            normal_score
        );
    }

    #[test]
    fn test_one_class_svm_rff() {
        let data = make_cluster(&[0.0, 0.0], 200, 1.0);
        let mut svm = OneClassSVM::new(200, 0.1);
        svm.fit(&data);

        let normal_score = svm.score(&[0.0, 0.0]);
        let outlier_score = svm.score(&[50.0, 50.0]);
        assert!(
            outlier_score > normal_score,
            "outlier_score={} should be > normal_score={}",
            outlier_score,
            normal_score
        );
    }

    #[test]
    fn test_lof_detector() {
        let data = make_cluster(&[0.0, 0.0], 100, 1.0);
        let mut lof = LOFDetector::new(20);
        lof.fit(&data);
        assert!(lof.trained);

        let normal_score = lof.score(&[0.0, 0.0]);
        let outlier_score = lof.score(&[50.0, 50.0]);
        assert!(
            outlier_score > normal_score,
            "outlier_score={} should be > normal_score={}",
            outlier_score,
            normal_score
        );
    }

    #[test]
    fn test_dbscan_clustering() {
        let mut data = make_cluster(&[0.0, 0.0], 50, 0.3);
        data.extend(make_cluster(&[10.0, 10.0], 50, 0.3));
        let mut dbscan = DBSCAN::new(0.8, 5);
        dbscan.fit(&data);
        assert!(dbscan.trained);

        // Should have at least 2 clusters
        let max_label = dbscan.labels.iter().copied().max().unwrap_or(-1);
        assert!(max_label >= 1, "Expected at least 2 clusters, got max_label={}", max_label);

        // Point far away should score higher
        let normal_score = dbscan.score(&[0.0, 0.0]);
        let outlier_score = dbscan.score(&[50.0, 50.0]);
        assert!(
            outlier_score > normal_score,
            "outlier_score={} should be > normal_score={}",
            outlier_score,
            normal_score
        );
    }

    #[test]
    fn test_production_ml_stack() {
        let data = make_cluster(&[0.0, 0.0], 300, 1.0);
        let mut stack = ProductionMLStack::new();
        assert!(!stack.is_trained());

        stack.fit(&data);
        assert!(stack.is_trained());

        let normal_score = stack.detect_anomaly(&[0.0, 0.0]);
        let outlier_score = stack.detect_anomaly(&[50.0, 50.0]);
        assert!(
            outlier_score > normal_score,
            "outlier_score={} should be > normal_score={}",
            outlier_score,
            normal_score
        );
    }

    #[test]
    fn test_empty_data_handling() {
        let empty: Vec<Vec<f64>> = Vec::new();

        // KdTree
        let tree = KdTree::build(&empty);
        assert!(tree.knn(&[0.0], 1).is_empty());
        assert!(tree.range_query(&[0.0], 1.0).is_empty());

        // IsolationForest
        let mut forest = IsolationForest::new(10, 256);
        forest.fit(&empty);
        let s = forest.score(&[0.0]);
        assert!((s - 0.5).abs() < 1e-10);

        // OneClassSVM
        let mut svm = OneClassSVM::new(50, 0.1);
        svm.fit(&empty);
        let s = svm.score(&[0.0]);
        assert!((s - 0.5).abs() < 1e-10);

        // LOFDetector
        let mut lof = LOFDetector::new(5);
        lof.fit(&empty);
        let s = lof.score(&[0.0]);
        assert!((s - 0.5).abs() < 1e-10);

        // DBSCAN
        let mut db = DBSCAN::new(0.5, 5);
        db.fit(&empty);
        let s = db.score(&[0.0]);
        assert!((s - 0.5).abs() < 1e-10);

        // ProductionMLStack
        let mut stack = ProductionMLStack::new();
        stack.fit(&empty);
        // Not trained because sub-models didn't train
        let s = stack.detect_anomaly(&[0.0]);
        assert!(s >= 0.0 && s <= 1.0);
    }
}
