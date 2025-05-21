use crate::{Node, NodeMut, NodeRef, Tree};

impl<T0: Node> Tree for (T0,) {
    fn get_ref<'a>(&'a self, path: &str) -> Option<NodeRef<'a>> {
        match path {
            "0" | "x" => Some(self.0.node_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<NodeMut<'a>> {
        match path {
            "0" | "x" => Some(self.0.node_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0"]
    }
}

impl<T0: Node, T1: Node> Tree for (T0, T1) {
    fn get_ref<'a>(&'a self, path: &str) -> Option<NodeRef<'a>> {
        match path {
            "0" | "x" => Some(self.0.node_ref()),
            "1" | "y" => Some(self.1.node_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<NodeMut<'a>> {
        match path {
            "0" | "x" => Some(self.0.node_mut()),
            "1" | "y" => Some(self.1.node_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1"]
    }
}

impl<T0: Node, T1: Node, T2: Node> Tree for (T0, T1, T2) {
    fn get_ref<'a>(&'a self, path: &str) -> Option<NodeRef<'a>> {
        match path {
            "0" | "x" => Some(self.0.node_ref()),
            "1" | "y" => Some(self.1.node_ref()),
            "2" | "z" => Some(self.2.node_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<NodeMut<'a>> {
        match path {
            "0" | "x" => Some(self.0.node_mut()),
            "1" | "y" => Some(self.1.node_mut()),
            "2" | "z" => Some(self.2.node_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1", "2"]
    }
}

impl<T0: Node, T1: Node, T2: Node, T3: Node> Tree for (T0, T1, T2, T3) {
    fn get_ref<'a>(&'a self, path: &str) -> Option<NodeRef<'a>> {
        match path {
            "0" | "x" => Some(self.0.node_ref()),
            "1" | "y" => Some(self.1.node_ref()),
            "2" | "z" => Some(self.2.node_ref()),
            "3" => Some(self.3.node_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<NodeMut<'a>> {
        match path {
            "0" | "x" => Some(self.0.node_mut()),
            "1" | "y" => Some(self.1.node_mut()),
            "2" | "z" => Some(self.2.node_mut()),
            "3" => Some(self.3.node_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1", "2", "3"]
    }
}

impl<T0: Node, T1: Node, T2: Node, T3: Node, T4: Node> Tree for (T0, T1, T2, T3, T4) {
    fn get_ref<'a>(&'a self, path: &str) -> Option<NodeRef<'a>> {
        match path {
            "0" | "x" => Some(self.0.node_ref()),
            "1" | "y" => Some(self.1.node_ref()),
            "2" | "z" => Some(self.2.node_ref()),
            "3" => Some(self.3.node_ref()),
            "4" => Some(self.4.node_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<NodeMut<'a>> {
        match path {
            "0" | "x" => Some(self.0.node_mut()),
            "1" | "y" => Some(self.1.node_mut()),
            "2" | "z" => Some(self.2.node_mut()),
            "3" => Some(self.3.node_mut()),
            "4" => Some(self.4.node_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1", "2", "3", "4"]
    }
}

impl<T0: Node, T1: Node, T2: Node, T3: Node, T4: Node, T5: Node> Tree for (T0, T1, T2, T3, T4, T5) {
    fn get_ref<'a>(&'a self, path: &str) -> Option<NodeRef<'a>> {
        match path {
            "0" | "x" => Some(self.0.node_ref()),
            "1" | "y" => Some(self.1.node_ref()),
            "2" | "z" => Some(self.2.node_ref()),
            "3" => Some(self.3.node_ref()),
            "4" => Some(self.4.node_ref()),
            "5" => Some(self.5.node_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<NodeMut<'a>> {
        match path {
            "0" | "x" => Some(self.0.node_mut()),
            "1" | "y" => Some(self.1.node_mut()),
            "2" | "z" => Some(self.2.node_mut()),
            "3" => Some(self.3.node_mut()),
            "4" => Some(self.4.node_mut()),
            "5" => Some(self.5.node_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1", "2", "3", "4", "5"]
    }
}

impl<T0: Node, T1: Node, T2: Node, T3: Node, T4: Node, T5: Node, T6: Node> Tree
    for (T0, T1, T2, T3, T4, T5, T6)
{
    fn get_ref<'a>(&'a self, path: &str) -> Option<NodeRef<'a>> {
        match path {
            "0" | "x" => Some(self.0.node_ref()),
            "1" | "y" => Some(self.1.node_ref()),
            "2" | "z" => Some(self.2.node_ref()),
            "3" => Some(self.3.node_ref()),
            "4" => Some(self.4.node_ref()),
            "5" => Some(self.5.node_ref()),
            "6" => Some(self.6.node_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<NodeMut<'a>> {
        match path {
            "0" | "x" => Some(self.0.node_mut()),
            "1" | "y" => Some(self.1.node_mut()),
            "2" | "z" => Some(self.2.node_mut()),
            "3" => Some(self.3.node_mut()),
            "4" => Some(self.4.node_mut()),
            "5" => Some(self.5.node_mut()),
            "6" => Some(self.6.node_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1", "2", "3", "4", "5", "6"]
    }
}

impl<T0: Node, T1: Node, T2: Node, T3: Node, T4: Node, T5: Node, T6: Node, T7: Node> Tree
    for (T0, T1, T2, T3, T4, T5, T6, T7)
{
    fn get_ref<'a>(&'a self, path: &str) -> Option<NodeRef<'a>> {
        match path {
            "0" | "x" => Some(self.0.node_ref()),
            "1" | "y" => Some(self.1.node_ref()),
            "2" | "z" => Some(self.2.node_ref()),
            "3" => Some(self.3.node_ref()),
            "4" => Some(self.4.node_ref()),
            "5" => Some(self.5.node_ref()),
            "6" => Some(self.6.node_ref()),
            "7" => Some(self.7.node_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<NodeMut<'a>> {
        match path {
            "0" | "x" => Some(self.0.node_mut()),
            "1" | "y" => Some(self.1.node_mut()),
            "2" | "z" => Some(self.2.node_mut()),
            "3" => Some(self.3.node_mut()),
            "4" => Some(self.4.node_mut()),
            "5" => Some(self.5.node_mut()),
            "6" => Some(self.6.node_mut()),
            "7" => Some(self.7.node_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1", "2", "3", "4", "5", "6", "7"]
    }
}

impl<T0: Node, T1: Node, T2: Node, T3: Node, T4: Node, T5: Node, T6: Node, T7: Node, T8: Node> Tree
    for (T0, T1, T2, T3, T4, T5, T6, T7, T8)
{
    fn get_ref<'a>(&'a self, path: &str) -> Option<NodeRef<'a>> {
        match path {
            "0" | "x" => Some(self.0.node_ref()),
            "1" | "y" => Some(self.1.node_ref()),
            "2" | "z" => Some(self.2.node_ref()),
            "3" => Some(self.3.node_ref()),
            "4" => Some(self.4.node_ref()),
            "5" => Some(self.5.node_ref()),
            "6" => Some(self.6.node_ref()),
            "7" => Some(self.7.node_ref()),
            "8" => Some(self.8.node_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<NodeMut<'a>> {
        match path {
            "0" | "x" => Some(self.0.node_mut()),
            "1" | "y" => Some(self.1.node_mut()),
            "2" | "z" => Some(self.2.node_mut()),
            "3" => Some(self.3.node_mut()),
            "4" => Some(self.4.node_mut()),
            "5" => Some(self.5.node_mut()),
            "6" => Some(self.6.node_mut()),
            "7" => Some(self.7.node_mut()),
            "8" => Some(self.8.node_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1", "2", "3", "4", "5", "6", "7", "8"]
    }
}

impl<
    T0: Node,
    T1: Node,
    T2: Node,
    T3: Node,
    T4: Node,
    T5: Node,
    T6: Node,
    T7: Node,
    T8: Node,
    T9: Node,
> Tree for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9)
{
    fn get_ref<'a>(&'a self, path: &str) -> Option<NodeRef<'a>> {
        match path {
            "0" | "x" => Some(self.0.node_ref()),
            "1" | "y" => Some(self.1.node_ref()),
            "2" | "z" => Some(self.2.node_ref()),
            "3" => Some(self.3.node_ref()),
            "4" => Some(self.4.node_ref()),
            "5" => Some(self.5.node_ref()),
            "6" => Some(self.6.node_ref()),
            "7" => Some(self.7.node_ref()),
            "8" => Some(self.8.node_ref()),
            "9" => Some(self.9.node_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<NodeMut<'a>> {
        match path {
            "0" | "x" => Some(self.0.node_mut()),
            "1" | "y" => Some(self.1.node_mut()),
            "2" | "z" => Some(self.2.node_mut()),
            "3" => Some(self.3.node_mut()),
            "4" => Some(self.4.node_mut()),
            "5" => Some(self.5.node_mut()),
            "6" => Some(self.6.node_mut()),
            "7" => Some(self.7.node_mut()),
            "8" => Some(self.8.node_mut()),
            "9" => Some(self.9.node_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]
    }
}
