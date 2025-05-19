use heapless::{String, Vec};

use crate::{EitherRef, LeafRef, Tree};

// Constants for maximum depth/length - adjust based on your requirements
pub const MAX_DEPTH: usize = 6; // Maximum recursion depth
pub const MAX_PATH_LEN: usize = 32; // Maximum path string length

/// The state of a single "level" of node iteration
struct Segment<'a> {
    // Reference to the node at this level
    node: &'a dyn Tree,
    // Current index in the node's entries
    index: usize,
}

#[derive(Clone, PartialEq, Debug)]
pub struct LeafPath(String<MAX_PATH_LEN>);

impl LeafPath {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    // Add segment to the path buffer
    fn push_segment(&mut self, segment: &str) -> bool {
        // Check if we have space for the segment + separator
        if self.0.len() + segment.len() + 1 <= MAX_PATH_LEN {
            let _ = self.0.push('.');
            let _ = self.0.push_str(segment);
            true
        } else {
            false
        }
    }

    // Remove the last segment from the path buffer
    fn pop_segment(&mut self) {
        // Find the last period and truncate there
        if let Some(pos) = self.0.rfind('.') {
            self.0.truncate(pos);
        }
    }
}

pub struct LeafIter<'a> {
    // Single path buffer that's modified during traversal
    path_buffer: LeafPath,
    // Stack stores only minimal data for traversal state
    stack: Vec<Segment<'a>, MAX_DEPTH>,
}

impl<'a> LeafIter<'a> {
    pub fn new(root: &'a dyn Tree, root_name: &str) -> Self {
        let mut path_buffer = LeafPath(String::new());
        _ = path_buffer.0.push_str(root_name);

        let mut stack = Vec::new();
        // Push the root node to begin traversal
        let _ = stack.push(Segment {
            node: root,
            index: 0,
        });

        Self { path_buffer, stack }
    }
}

impl<'a> Iterator for LeafIter<'a> {
    type Item = (LeafPath, LeafRef<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let segment = self.stack.last_mut()?;
            let entries = segment.node.entries();

            // Check if we've processed all entries in the current node
            if segment.index >= entries.len() {
                // We're done with this node
                self.stack.pop()?;

                // Don't pop path segment for the root level
                if !self.stack.is_empty() {
                    self.path_buffer.pop_segment();
                }

                continue;
            }

            // Get the next entry to process
            let entry_name = entries[segment.index];
            segment.index += 1;

            match segment.node.get_ref(entry_name)? {
                EitherRef::Leaf(leaf_ref) => {
                    // Found a leaf - temporarily add to path and return
                    self.path_buffer.push_segment(entry_name);

                    // Create a copy of the current path for the return value
                    let result_path = self.path_buffer.clone();

                    // Remove the temporary segment from our buffer
                    self.path_buffer.pop_segment();

                    return Some((result_path, leaf_ref));
                }
                EitherRef::Tree(node_ref) => {
                    // Add this segment to the path
                    if self.path_buffer.push_segment(entry_name) {
                        // Push this node for traversal
                        let _ = self.stack.push(Segment {
                            node: node_ref.0,
                            index: 0,
                        });
                    }
                }
            }
        }
    }
}
