//! Commit graph computation for visualization

use crate::error::GitError;
use crate::repository::Repository;
use log::info;
use std::collections::HashMap;

/// Visual layout data for a single commit in the graph view
#[derive(Debug, Clone)]
pub struct GraphNode {
    /// Reference to commit hash
    pub commit_id: String,
    /// Column index for this commit's position
    pub lane: u32,
    /// Edges connecting to parent commits
    pub parent_edges: Vec<GraphEdge>,
    /// Whether this commit has multiple parents (merge commit)
    pub is_merge: bool,
}

/// Visual edge connecting two commits in the graph
#[derive(Debug, Clone)]
pub struct GraphEdge {
    /// Source lane (child commit)
    pub from_lane: u32,
    /// Target lane (parent commit)
    pub to_lane: u32,
    /// Edge rendering type
    pub edge_type: EdgeType,
    /// Color palette index for branch coloring
    pub color_index: u8,
}

/// Edge rendering type
#[derive(Debug, Clone, PartialEq)]
pub enum EdgeType {
    /// Straight line in same lane
    Direct,
    /// Cross-lane merge edge
    Merge,
    /// Branch fork edge
    Fork,
}

/// Reference label attached to a commit
#[derive(Debug, Clone)]
pub struct RefLabel {
    /// Reference display name
    pub name: String,
    /// Reference type
    pub ref_type: RefType,
    /// Whether this is the current HEAD ref
    pub is_current: bool,
}

/// Type of reference
#[derive(Debug, Clone, PartialEq)]
pub enum RefType {
    LocalBranch,
    RemoteBranch,
    Tag,
    Head,
}

/// Compute graph layout for a list of commits
///
/// Uses a lane-based algorithm:
/// 1. Walk commits in topological order
/// 2. Assign each branch tip to the leftmost available lane
/// 3. Merge points consume the child's lane; parent continues in its lane
/// 4. Track active lanes at each row for edge rendering
pub fn compute_graph(
    repo: &Repository,
    commit_ids: &[String],
) -> Result<Vec<GraphNode>, GitError> {
    info!("Computing graph layout for {} commits", commit_ids.len());

    let repo_lock = repo.inner.read().unwrap();

    // Map commit ID → index for quick lookup
    let id_to_index: HashMap<String, usize> = commit_ids
        .iter()
        .enumerate()
        .map(|(i, id)| (id.clone(), i))
        .collect();

    // Track which lane each commit occupies
    let mut active_lanes: Vec<Option<String>> = Vec::new(); // lane index → commit_id occupying it
    let mut nodes = Vec::with_capacity(commit_ids.len());
    let mut color_counter: u8 = 0;
    let mut commit_lane_map: HashMap<String, u32> = HashMap::new();
    let mut commit_color_map: HashMap<String, u8> = HashMap::new();

    for commit_id in commit_ids {
        let oid = git2::Oid::from_str(commit_id).map_err(|e| GitError::CommitNotFound {
            id: format!("{}: {}", commit_id, e),
        })?;

        let commit = repo_lock
            .find_commit(oid)
            .map_err(|_| GitError::CommitNotFound {
                id: commit_id.clone(),
            })?;

        let parent_ids: Vec<String> = commit.parent_ids().map(|id| id.to_string()).collect();
        let is_merge = parent_ids.len() > 1;

        // Find or assign lane for this commit
        let lane = if let Some(&l) = commit_lane_map.get(commit_id) {
            l
        } else {
            // Assign to the leftmost free lane
            let free_lane = active_lanes
                .iter()
                .position(|l| l.is_none())
                .unwrap_or_else(|| {
                    active_lanes.push(None);
                    active_lanes.len() - 1
                });
            active_lanes[free_lane] = Some(commit_id.clone());
            commit_lane_map.insert(commit_id.clone(), free_lane as u32);
            commit_color_map.insert(commit_id.clone(), color_counter);
            color_counter = color_counter.wrapping_add(1);
            free_lane as u32
        };

        let color = *commit_color_map.get(commit_id).unwrap_or(&0);

        // Free this commit's lane
        if (lane as usize) < active_lanes.len() {
            active_lanes[lane as usize] = None;
        }

        // Assign lanes to parents
        let mut parent_edges = Vec::new();

        for (pi, parent_id) in parent_ids.iter().enumerate() {
            // Only process parents that appear in our commit list
            if !id_to_index.contains_key(parent_id) {
                continue;
            }

            let parent_lane = if pi == 0 {
                // First parent continues in same lane
                if (lane as usize) < active_lanes.len() && active_lanes[lane as usize].is_none() {
                    active_lanes[lane as usize] = Some(parent_id.clone());
                    commit_lane_map.insert(parent_id.clone(), lane);
                    commit_color_map.insert(parent_id.clone(), color);
                    lane
                } else {
                    let free = active_lanes
                        .iter()
                        .position(|l| l.is_none())
                        .unwrap_or_else(|| {
                            active_lanes.push(None);
                            active_lanes.len() - 1
                        });
                    active_lanes[free] = Some(parent_id.clone());
                    commit_lane_map.insert(parent_id.clone(), free as u32);
                    commit_color_map.insert(parent_id.clone(), color);
                    free as u32
                }
            } else {
                // Additional parents (merge) get new lanes
                if let Some(&existing) = commit_lane_map.get(parent_id) {
                    existing
                } else {
                    let free = active_lanes
                        .iter()
                        .position(|l| l.is_none())
                        .unwrap_or_else(|| {
                            active_lanes.push(None);
                            active_lanes.len() - 1
                        });
                    active_lanes[free] = Some(parent_id.clone());
                    let new_color = color_counter;
                    color_counter = color_counter.wrapping_add(1);
                    commit_lane_map.insert(parent_id.clone(), free as u32);
                    commit_color_map.insert(parent_id.clone(), new_color);
                    free as u32
                }
            };

            let edge_type = if parent_lane == lane {
                EdgeType::Direct
            } else if is_merge && pi > 0 {
                EdgeType::Merge
            } else {
                EdgeType::Fork
            };

            parent_edges.push(GraphEdge {
                from_lane: lane,
                to_lane: parent_lane,
                edge_type,
                color_index: *commit_color_map.get(parent_id).unwrap_or(&0),
            });
        }

        nodes.push(GraphNode {
            commit_id: commit_id.clone(),
            lane,
            parent_edges,
            is_merge,
        });
    }

    info!("Graph computed: {} nodes, max {} lanes", nodes.len(), active_lanes.len());
    Ok(nodes)
}

/// Compute ref labels (branches and tags) for each commit
pub fn compute_ref_labels(repo: &Repository) -> Result<HashMap<String, Vec<RefLabel>>, GitError> {
    info!("Computing ref labels");

    let repo_lock = repo.inner.read().unwrap();
    let mut labels: HashMap<String, Vec<RefLabel>> = HashMap::new();

    // Get current HEAD
    let head_target = repo_lock
        .head()
        .ok()
        .and_then(|h| h.target())
        .map(|oid| oid.to_string());

    let head_branch = repo_lock
        .head()
        .ok()
        .and_then(|h| {
            if h.is_branch() {
                h.shorthand().map(|s| s.to_string())
            } else {
                None
            }
        });

    // Iterate all references
    let refs = repo_lock.references().map_err(|e| GitError::OperationFailed {
        operation: "compute_ref_labels".to_string(),
        details: format!("Failed to enumerate references: {}", e),
    })?;

    for reference in refs.flatten() {
        let name = match reference.name() {
            Some(n) => n.to_string(),
            None => continue,
        };

        let target = reference
            .peel_to_commit()
            .ok()
            .map(|c| c.id().to_string());

        let target_id = match target {
            Some(id) => id,
            None => continue,
        };

        let (display_name, ref_type) = if name.starts_with("refs/heads/") {
            (
                name.strip_prefix("refs/heads/").unwrap_or(&name).to_string(),
                RefType::LocalBranch,
            )
        } else if name.starts_with("refs/remotes/") {
            (
                name.strip_prefix("refs/remotes/").unwrap_or(&name).to_string(),
                RefType::RemoteBranch,
            )
        } else if name.starts_with("refs/tags/") {
            (
                name.strip_prefix("refs/tags/").unwrap_or(&name).to_string(),
                RefType::Tag,
            )
        } else {
            continue;
        };

        let is_current = match ref_type {
            RefType::LocalBranch => head_branch.as_deref() == Some(&display_name),
            _ => false,
        };

        labels
            .entry(target_id)
            .or_default()
            .push(RefLabel {
                name: display_name,
                ref_type,
                is_current,
            });
    }

    // Add HEAD label
    if let Some(head_id) = head_target {
        if head_branch.is_none() {
            // Detached HEAD
            labels
                .entry(head_id)
                .or_default()
                .push(RefLabel {
                    name: "HEAD".to_string(),
                    ref_type: RefType::Head,
                    is_current: true,
                });
        }
    }

    info!("Ref labels computed for {} commits", labels.len());
    Ok(labels)
}
