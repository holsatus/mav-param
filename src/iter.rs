use heapless::Vec;

use crate::{Error, NodeRef, Parameter};

/// Maximum "recursion" depth
pub const MAX_STACK_DEPTH: usize = 5;

/// The state of a single "level" of tree iteration
struct Segment<'a> {
    // Reference to the tree at this level
    node_ref: NodeRef<'a>,
    // Current index in the tree's entries
    index: usize,
}

/// An iterator that traverses a parameter tree and yields each leaf parameter.
///
/// This iterator performs a depth-first traversal of the parameter tree, building
/// parameter identifiers using dot notation (e.g., "sys.sub.param").
/// It handles errors related to path length and tree depth limitations.
pub struct ParamIter<'a> {
    // Single path buffer that's modified during traversal
    ident_buffer: crate::ident::Ident,
    // Stack stores only minimal data for traversal state
    stack: Vec<Segment<'a>, MAX_STACK_DEPTH>,
}

impl<'a> ParamIter<'a> {
    /// Creates a new parameter iterator starting at the given tree.
    ///
    /// Note: A tree is never aware of its parents, so they are not included in the path.
    pub fn new(node_ref: NodeRef<'a>, name: Option<&str>) -> Self {
        let mut ident_buffer = crate::ident::Ident::new();

        if let Some(name) = name {
            _ = ident_buffer.push_entry(name);
        }

        // Push the tree root to begin traversal
        let mut stack = Vec::new();
        let _ = stack.push(Segment { node_ref, index: 0 });

        Self {
            ident_buffer,
            stack,
        }
    }
}

impl ParamIter<'_> {}

impl Iterator for ParamIter<'_> {
    type Item = Result<Parameter, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let segment = self.stack.last_mut()?;

            match segment.node_ref {
                NodeRef::None => {
                    self.stack.pop()?;
                    self.ident_buffer.pop_entry();
                    continue;
                }
                NodeRef::Tree(tree) => {
                    let entries = tree.entries();

                    // Check if we've processed all entries in the current tree
                    if segment.index >= entries.len() {
                        // We're done with this node
                        self.stack.pop()?;

                        // Don't pop path segment for the root level
                        if !self.stack.is_empty() {
                            self.ident_buffer.pop_entry();
                        }

                        continue;
                    }

                    // Get the next entry to process
                    let entry_name = entries[segment.index];
                    segment.index += 1;

                    // If no node is returned, it is because it is conditionally disabled
                    let Some(node_ref) = tree.get_ref(entry_name) else {
                        continue;
                    };

                    // Add this segment to the path (temporarily)
                    if !self.ident_buffer.push_entry(entry_name) {
                        return Some(Err(Error::PathTooLong(
                            self.ident_buffer.clone(),
                            entry_name,
                        )));
                    }

                    if self.stack.push(Segment { node_ref, index: 0 }).is_err() {
                        return Some(Err(Error::DepthTooBig(
                            self.ident_buffer.clone(),
                            entry_name,
                        )));
                    }
                }
                NodeRef::Enum(union) => {
                    let entry_name = "#";

                    // Add this enum segment to the path (temporarily)
                    if !self.ident_buffer.push_entry(entry_name) {
                        return Some(Err(Error::PathTooLong(
                            self.ident_buffer.clone(),
                            entry_name,
                        )));
                    }

                    // Create a copy of the current path for the return value
                    let ident = self.ident_buffer.clone();

                    // Remove the temporary segment from our buffer
                    self.ident_buffer.pop_entry();

                    // Set the current node as the active variant
                    segment.node_ref = union.active_node_ref();

                    // Return the discriminant as the value
                    return Some(Ok(Parameter {
                        ident,
                        value: union.get(),
                    }));
                }
                // Maybe just handle it like
                NodeRef::Leaf(value) => {
                    // Create a copy of the current path for the return value
                    let ident = self.ident_buffer.clone();

                    // Remove the temporary segment from our buffer
                    self.ident_buffer.pop_entry();
                    self.stack.pop();

                    return Some(Ok(Parameter {
                        ident,
                        value: value.get(),
                    }));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{self as mav_param, Value};
    use mav_param::Error;
    use mav_param::{Tree, param_iter_named};

    #[test]
    fn basic_iteration() {
        #[derive(Tree)]
        struct TestParams {
            #[tree(rename = "sub")]
            subtree: SubTree,
            value1: u8,
            value2: i16,
            float_val: f32,
        }

        #[derive(Tree)]
        struct SubTree {
            leaf1: u32,
            leaf2: f32,
            #[tree(rename = "deep")]
            deeper: DeepSubTree,
        }

        #[derive(Tree)]
        struct DeepSubTree {
            val: i8,
        }

        let params = TestParams {
            subtree: SubTree {
                leaf1: 42,
                leaf2: 3.14,
                deeper: DeepSubTree { val: -5 },
            },
            value1: 10,
            value2: -100,
            float_val: 2.718,
        };

        // Collect all parameters into a vector
        let results: Vec<_> = param_iter_named(&params, "test")
            .filter_map(Result::ok)
            .take(20)
            .collect();

        // Check we got the expected number of parameters (test.sub.deep.val should fail)
        assert_eq!(results.len(), 5, "Should iterate over 5 leaf values");

        // Check specific parameters
        let expected_params = vec![
            ("test.sub.leaf1", Value::U32(42)),
            ("test.sub.leaf2", Value::F32(3.14)),
            ("test.value1", Value::U8(10)),
            ("test.value2", Value::I16(-100)),
            ("test.float_val", Value::F32(2.718)),
        ];

        for (param, (expected_path, expected_value)) in results.iter().zip(expected_params.iter()) {
            assert_eq!(param.ident.as_str(), *expected_path);

            match (&param.value, expected_value) {
                (Value::U8(a), Value::U8(b)) => assert_eq!(a, b),
                (Value::I8(a), Value::I8(b)) => assert_eq!(a, b),
                (Value::U16(a), Value::U16(b)) => assert_eq!(a, b),
                (Value::I16(a), Value::I16(b)) => assert_eq!(a, b),
                (Value::U32(a), Value::U32(b)) => assert_eq!(a, b),
                (Value::I32(a), Value::I32(b)) => assert_eq!(a, b),
                (Value::F32(a), Value::F32(b)) => assert!((a - b).abs() < f32::EPSILON),
                _ => panic!("Value type mismatch"),
            }
        }
    }

    #[test]
    fn max_depth_error() {
        #[derive(Tree)]
        struct MaxDepthTree {
            l1: MaxDepthL1,
        }

        #[derive(Tree)]
        struct MaxDepthL1 {
            l2: MaxDepthL2,
        }

        #[derive(Tree)]
        struct MaxDepthL2 {
            l3: MaxDepthL3,
        }

        #[derive(Tree)]
        struct MaxDepthL3 {
            l4: MaxDepthL4,
        }

        #[derive(Tree)]
        struct MaxDepthL4 {
            l5: MaxDepthL5,
        }

        #[derive(Tree)]
        struct MaxDepthL5 {
            l6: u8,
        }

        // Create a tree that exceeds max depth
        let deep_tree = MaxDepthTree {
            l1: MaxDepthL1 {
                l2: MaxDepthL2 {
                    l3: MaxDepthL3 {
                        l4: MaxDepthL4 {
                            l5: MaxDepthL5 { l6: 42 },
                        },
                    },
                },
            },
        };

        // Try to iterate - should encounter DepthTooBig error
        let mut iter = param_iter_named(&deep_tree, "d");
        let mut found_depth_error = false;

        // We should be able to traverse until we hit the max depth
        while let Some(result) = iter.next() {
            if let Err(Error::DepthTooBig(_, _)) = result {
                found_depth_error = true;
                break;
            }
        }

        assert!(found_depth_error, "Should encounter a DepthTooBig error");
    }
}
