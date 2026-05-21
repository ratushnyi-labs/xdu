//! Arena-backed scan tree.
//!
//! Hierarchy is stored as a flat `Vec<Node>` indexed by [`NodeId`] (`u32`).
//! Sibling lists are intrusive (each node carries `next_sibling`), the parent
//! holds `first_child`. Aggregation is iterative (post-order via an explicit
//! stack) to remain safe on very deep trees.

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct NodeId(u32);

impl NodeId {
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum NodeKind {
    Dir,
    File,
    Symlink,
    Other,
}

#[derive(Debug)]
pub struct Node {
    pub kind: NodeKind,
    pub name: Box<str>,
    pub size: u64,
    pub alloc: u64,
    pub entries: u64,
    pub error: bool,
    /// Unix-epoch seconds of the entry's last access time.
    /// `0` means "unknown" (e.g., metadata read failed or the FS does not
    /// expose access time). Aggregation does not propagate this — it is a
    /// per-entry attribute, displayed only on rows.
    pub accessed_unix: u64,
    /// `true` when the node's `size` / `alloc` / `entries` are final.
    /// Files / symlinks / other are finalized at insert. Directories are
    /// finalized only after the scanner has fully processed their subtree
    /// (BUG-002). The TUI renders `???` for `!finalized` rows.
    pub finalized: bool,
    pub parent: Option<NodeId>,
    pub first_child: Option<NodeId>,
    pub next_sibling: Option<NodeId>,
}

#[derive(Debug)]
pub struct Tree {
    nodes: Vec<Node>,
    root: NodeId,
    overflow: bool,
    size_overflow: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SortKey {
    SizeDesc,
    SizeAsc,
    NameAsc,
    NameDesc,
    CountDesc,
    CountAsc,
}

#[derive(Debug, Eq, PartialEq)]
pub enum TreeError {
    OutOfCapacity,
    InvalidParent,
}

impl Tree {
    pub fn new(root_kind: NodeKind, root_name: impl Into<Box<str>>) -> Self {
        let finalized = !matches!(root_kind, NodeKind::Dir);
        let nodes = vec![Node {
            kind: root_kind,
            name: root_name.into(),
            size: 0,
            alloc: 0,
            entries: 1,
            error: false,
            accessed_unix: 0,
            finalized,
            parent: None,
            first_child: None,
            next_sibling: None,
        }];
        Self {
            nodes,
            root: NodeId(0),
            overflow: false,
            size_overflow: false,
        }
    }

    pub fn root(&self) -> NodeId {
        self.root
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn overflow(&self) -> bool {
        self.overflow
    }

    pub fn size_overflow(&self) -> bool {
        self.size_overflow
    }

    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.index())
    }

    pub fn add_child(
        &mut self,
        parent: NodeId,
        kind: NodeKind,
        name: impl Into<Box<str>>,
        size: u64,
        alloc: u64,
    ) -> Result<NodeId, TreeError> {
        if parent.index() >= self.nodes.len() {
            return Err(TreeError::InvalidParent);
        }
        if self.nodes.len() >= u32::MAX as usize {
            self.overflow = true;
            return Err(TreeError::OutOfCapacity);
        }
        let id = NodeId(self.nodes.len() as u32);
        let prev_first = self.nodes[parent.index()].first_child;
        let finalized = !matches!(kind, NodeKind::Dir);
        self.nodes.push(Node {
            kind,
            name: name.into(),
            size,
            alloc,
            entries: 1,
            error: false,
            accessed_unix: 0,
            finalized,
            parent: Some(parent),
            first_child: None,
            next_sibling: prev_first,
        });
        self.nodes[parent.index()].first_child = Some(id);
        Ok(id)
    }

    pub fn mark_error(&mut self, id: NodeId) {
        if let Some(n) = self.nodes.get_mut(id.index()) {
            n.error = true;
        }
    }

    /// Set the last-access timestamp (Unix-epoch seconds) on an existing node.
    /// `0` means unknown. Called by the scanner after [`Self::add_child`].
    pub fn set_accessed(&mut self, id: NodeId, secs: u64) {
        if let Some(n) = self.nodes.get_mut(id.index()) {
            n.accessed_unix = secs;
        }
    }

    /// Mark a node finalized (or not). Called by the scanner after a directory's
    /// subtree has been fully processed (BUG-002). The TUI uses this flag to
    /// decide whether to render the size column or `???`.
    pub fn set_finalized(&mut self, id: NodeId, val: bool) {
        if let Some(n) = self.nodes.get_mut(id.index()) {
            n.finalized = val;
        }
    }

    /// Mark every node in the tree as finalized. Test-only convenience for the
    /// snapshot harness (TASK-007-F): a hand-built fixture tree has all data
    /// in place from the start, so we want all rows to render real numbers
    /// rather than the mid-scan spinner.
    pub fn finalize_all(&mut self) {
        for n in &mut self.nodes {
            n.finalized = true;
        }
    }

    /// Walk the parent chain from `from` (excluding `from` itself) and
    /// saturating-add `(size, alloc, entries)` to every ancestor. Used by the
    /// async scanner for eager aggregation so partial sums are visible mid-scan.
    pub fn bubble_up_add(&mut self, from: NodeId, size: u64, alloc: u64, entries: u64) {
        let Some(start) = self.nodes.get(from.index()) else {
            return;
        };
        let mut p = start.parent;
        while let Some(parent) = p {
            let Some(n) = self.nodes.get_mut(parent.index()) else {
                break;
            };
            n.size = n.size.saturating_add(size);
            n.alloc = n.alloc.saturating_add(alloc);
            n.entries = n.entries.saturating_add(entries);
            if n.size == u64::MAX || n.alloc == u64::MAX || n.entries == u64::MAX {
                self.size_overflow = true;
            }
            p = n.parent;
        }
    }

    pub fn children(&self, parent: NodeId) -> ChildrenIter<'_> {
        ChildrenIter {
            tree: self,
            next: self.nodes.get(parent.index()).and_then(|n| n.first_child),
        }
    }

    pub fn path(&self, id: NodeId) -> Vec<&str> {
        let mut parts: Vec<&str> = Vec::new();
        let mut cur = Some(id);
        while let Some(c) = cur {
            let Some(n) = self.nodes.get(c.index()) else {
                break;
            };
            parts.push(n.name.as_ref());
            cur = n.parent;
        }
        parts.reverse();
        parts
    }

    pub fn sort(&mut self, parent: NodeId, key: SortKey) {
        let ids: Vec<NodeId> = self.children(parent).collect();
        if ids.is_empty() {
            return;
        }
        let mut sorted = ids;
        sorted.sort_by(|&a, &b| Self::compare(&self.nodes, a, b, key));
        self.nodes[parent.index()].first_child = Some(sorted[0]);
        for i in 0..sorted.len() {
            let next = sorted.get(i + 1).copied();
            self.nodes[sorted[i].index()].next_sibling = next;
        }
    }

    fn compare(nodes: &[Node], a: NodeId, b: NodeId, key: SortKey) -> std::cmp::Ordering {
        let na = &nodes[a.index()];
        let nb = &nodes[b.index()];
        let tie = na.name.cmp(&nb.name);
        match key {
            SortKey::SizeDesc => nb.size.cmp(&na.size).then(tie),
            SortKey::SizeAsc => na.size.cmp(&nb.size).then(tie),
            SortKey::NameAsc => na.name.cmp(&nb.name),
            SortKey::NameDesc => nb.name.cmp(&na.name),
            SortKey::CountDesc => nb.entries.cmp(&na.entries).then(tie),
            SortKey::CountAsc => na.entries.cmp(&nb.entries).then(tie),
        }
    }

    pub fn aggregate(&mut self) {
        self.aggregate_subtree(self.root);
    }

    pub fn aggregate_subtree(&mut self, start: NodeId) {
        let mut stack: Vec<(NodeId, bool)> = Vec::with_capacity(64);
        stack.push((start, false));
        while let Some((id, visited)) = stack.pop() {
            if !visited {
                stack.push((id, true));
                let mut child = self.nodes[id.index()].first_child;
                while let Some(cid) = child {
                    stack.push((cid, false));
                    child = self.nodes[cid.index()].next_sibling;
                }
                continue;
            }
            self.fold_node(id);
        }
    }

    fn fold_node(&mut self, id: NodeId) {
        let kind = self.nodes[id.index()].kind;
        let (size, alloc, entries, err) = self.sum_children(id);
        if size == u64::MAX || alloc == u64::MAX || entries == u64::MAX {
            self.size_overflow = true;
        }
        let n = &mut self.nodes[id.index()];
        if matches!(kind, NodeKind::Dir) {
            n.size = size;
            n.alloc = alloc;
            n.entries = entries;
        }
        if err {
            n.error = true;
        }
    }

    fn sum_children(&self, parent: NodeId) -> (u64, u64, u64, bool) {
        let mut size = 0_u64;
        let mut alloc = 0_u64;
        let mut entries = 1_u64;
        let mut err = false;
        for cid in self.children(parent) {
            let c = &self.nodes[cid.index()];
            size = size.saturating_add(c.size);
            alloc = alloc.saturating_add(c.alloc);
            entries = entries.saturating_add(c.entries);
            if c.error {
                err = true;
            }
        }
        (size, alloc, entries, err)
    }
}

pub struct ChildrenIter<'a> {
    tree: &'a Tree,
    next: Option<NodeId>,
}

impl Iterator for ChildrenIter<'_> {
    type Item = NodeId;
    fn next(&mut self) -> Option<NodeId> {
        let cur = self.next?;
        self.next = self.tree.nodes[cur.index()].next_sibling;
        Some(cur)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(tree: &Tree, parent: NodeId) -> Vec<String> {
        tree.children(parent)
            .map(|id| tree.get(id).unwrap().name.to_string())
            .collect()
    }

    #[test]
    fn new_tree_has_root_only() {
        let t = Tree::new(NodeKind::Dir, "/");
        assert_eq!(t.len(), 1);
        let root = t.get(t.root()).unwrap();
        assert_eq!(root.name.as_ref(), "/");
        assert_eq!(root.entries, 1);
        assert_eq!(root.size, 0);
        assert!(root.first_child.is_none());
    }

    #[test]
    fn add_child_increments_id_and_links() {
        let mut t = Tree::new(NodeKind::Dir, "/");
        let root = t.root();
        let a = t.add_child(root, NodeKind::Dir, "a", 0, 0).unwrap();
        let b = t.add_child(root, NodeKind::File, "b", 10, 10).unwrap();
        assert_eq!(a.index(), 1);
        assert_eq!(b.index(), 2);
        let kids: Vec<_> = t.children(root).collect();
        assert_eq!(kids.len(), 2);
    }

    #[test]
    fn add_child_invalid_parent_is_rejected() {
        let mut t = Tree::new(NodeKind::Dir, "/");
        let bad = NodeId(99);
        let err = t.add_child(bad, NodeKind::File, "x", 0, 0).unwrap_err();
        assert_eq!(err, TreeError::InvalidParent);
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn aggregate_dir_sums_children() {
        let mut t = Tree::new(NodeKind::Dir, "/");
        let root = t.root();
        let a = t.add_child(root, NodeKind::Dir, "a", 0, 0).unwrap();
        t.add_child(a, NodeKind::File, "f1", 100, 4096).unwrap();
        t.add_child(a, NodeKind::File, "f2", 200, 4096).unwrap();
        t.aggregate();
        let na = t.get(a).unwrap();
        assert_eq!(na.size, 300);
        assert_eq!(na.alloc, 8192);
        assert_eq!(na.entries, 3);
        let nr = t.get(root).unwrap();
        assert_eq!(nr.size, 300);
        assert_eq!(nr.entries, 4);
    }

    #[test]
    fn aggregate_handles_deep_chain() {
        let mut t = Tree::new(NodeKind::Dir, "/");
        let mut cur = t.root();
        const DEPTH: usize = 100_000;
        for i in 0..DEPTH {
            cur = t
                .add_child(cur, NodeKind::Dir, format!("d{i}"), 0, 0)
                .unwrap();
        }
        t.add_child(cur, NodeKind::File, "leaf", 7, 7).unwrap();
        t.aggregate();
        assert_eq!(t.get(t.root()).unwrap().size, 7);
        assert_eq!(t.get(t.root()).unwrap().entries as usize, DEPTH + 2);
    }

    #[test]
    fn aggregate_saturates_on_overflow() {
        let mut t = Tree::new(NodeKind::Dir, "/");
        let root = t.root();
        t.add_child(root, NodeKind::File, "a", u64::MAX, 0).unwrap();
        t.add_child(root, NodeKind::File, "b", 1, 0).unwrap();
        t.aggregate();
        assert_eq!(t.get(root).unwrap().size, u64::MAX);
        assert!(t.size_overflow());
    }

    #[test]
    fn sort_size_desc() {
        let mut t = Tree::new(NodeKind::Dir, "/");
        let root = t.root();
        t.add_child(root, NodeKind::File, "small", 100, 100)
            .unwrap();
        t.add_child(root, NodeKind::File, "big", 1000, 1000)
            .unwrap();
        t.add_child(root, NodeKind::File, "medium", 500, 500)
            .unwrap();
        t.sort(root, SortKey::SizeDesc);
        assert_eq!(names(&t, root), vec!["big", "medium", "small"]);
    }

    #[test]
    fn sort_size_asc() {
        let mut t = Tree::new(NodeKind::Dir, "/");
        let root = t.root();
        t.add_child(root, NodeKind::File, "big", 1000, 0).unwrap();
        t.add_child(root, NodeKind::File, "small", 1, 0).unwrap();
        t.add_child(root, NodeKind::File, "mid", 500, 0).unwrap();
        t.sort(root, SortKey::SizeAsc);
        assert_eq!(names(&t, root), vec!["small", "mid", "big"]);
    }

    #[test]
    fn sort_name_asc_and_desc() {
        let mut t = Tree::new(NodeKind::Dir, "/");
        let root = t.root();
        t.add_child(root, NodeKind::File, "c", 0, 0).unwrap();
        t.add_child(root, NodeKind::File, "a", 0, 0).unwrap();
        t.add_child(root, NodeKind::File, "b", 0, 0).unwrap();
        t.sort(root, SortKey::NameAsc);
        assert_eq!(names(&t, root), vec!["a", "b", "c"]);
        t.sort(root, SortKey::NameDesc);
        assert_eq!(names(&t, root), vec!["c", "b", "a"]);
    }

    #[test]
    fn sort_count_desc_uses_entries() {
        let mut t = Tree::new(NodeKind::Dir, "/");
        let root = t.root();
        let a = t.add_child(root, NodeKind::Dir, "a", 0, 0).unwrap();
        let b = t.add_child(root, NodeKind::Dir, "b", 0, 0).unwrap();
        t.add_child(a, NodeKind::File, "f1", 0, 0).unwrap();
        for i in 0..5 {
            t.add_child(b, NodeKind::File, format!("g{i}"), 0, 0)
                .unwrap();
        }
        t.aggregate();
        t.sort(root, SortKey::CountDesc);
        assert_eq!(names(&t, root), vec!["b", "a"]);
    }

    #[test]
    fn sort_size_desc_breaks_ties_by_name_asc() {
        let mut t = Tree::new(NodeKind::Dir, "/");
        let root = t.root();
        t.add_child(root, NodeKind::File, "b", 100, 0).unwrap();
        t.add_child(root, NodeKind::File, "a", 100, 0).unwrap();
        t.add_child(root, NodeKind::File, "c", 100, 0).unwrap();
        t.sort(root, SortKey::SizeDesc);
        assert_eq!(names(&t, root), vec!["a", "b", "c"]);
    }

    #[test]
    fn children_iterator_visits_all_in_order() {
        let mut t = Tree::new(NodeKind::Dir, "/");
        let root = t.root();
        let a = t.add_child(root, NodeKind::File, "a", 0, 0).unwrap();
        let b = t.add_child(root, NodeKind::File, "b", 0, 0).unwrap();
        let c = t.add_child(root, NodeKind::File, "c", 0, 0).unwrap();
        t.sort(root, SortKey::NameAsc);
        let kids: Vec<NodeId> = t.children(root).collect();
        assert_eq!(kids, vec![a, b, c]);
    }

    #[test]
    fn path_returns_breadcrumb_root_first() {
        let mut t = Tree::new(NodeKind::Dir, "/");
        let root = t.root();
        let a = t.add_child(root, NodeKind::Dir, "a", 0, 0).unwrap();
        let b = t.add_child(a, NodeKind::Dir, "b", 0, 0).unwrap();
        let c = t.add_child(b, NodeKind::File, "c.txt", 0, 0).unwrap();
        let p = t.path(c);
        assert_eq!(p, vec!["/", "a", "b", "c.txt"]);
    }

    #[test]
    fn mark_error_propagates_via_aggregate() {
        let mut t = Tree::new(NodeKind::Dir, "/");
        let root = t.root();
        let a = t.add_child(root, NodeKind::Dir, "a", 0, 0).unwrap();
        let b = t.add_child(a, NodeKind::Dir, "b", 0, 0).unwrap();
        let leaf = t.add_child(b, NodeKind::File, "x", 0, 0).unwrap();
        t.mark_error(leaf);
        t.aggregate();
        assert!(t.get(leaf).unwrap().error);
        assert!(t.get(b).unwrap().error);
        assert!(t.get(a).unwrap().error);
        assert!(t.get(root).unwrap().error);
    }

    #[test]
    fn set_accessed_records_timestamp_per_node() {
        let mut t = Tree::new(NodeKind::Dir, "/");
        let root = t.root();
        let a = t.add_child(root, NodeKind::File, "a", 1, 1).unwrap();
        let b = t.add_child(root, NodeKind::File, "b", 1, 1).unwrap();
        assert_eq!(t.get(a).unwrap().accessed_unix, 0);
        t.set_accessed(a, 1_700_000_000);
        assert_eq!(t.get(a).unwrap().accessed_unix, 1_700_000_000);
        // Other nodes unaffected.
        assert_eq!(t.get(b).unwrap().accessed_unix, 0);
    }

    #[test]
    fn set_accessed_on_unknown_id_is_no_op() {
        let mut t = Tree::new(NodeKind::Dir, "/");
        t.set_accessed(NodeId(99), 1_700_000_000);
        // No panic; root unaffected.
        assert_eq!(t.get(t.root()).unwrap().accessed_unix, 0);
    }

    #[test]
    fn test_sec_uc002_get_unknown_id_returns_none() {
        let t = Tree::new(NodeKind::Dir, "/");
        assert!(t.get(NodeId(99)).is_none());
    }

    #[test]
    fn test_sec_uc002_aggregate_does_not_panic_on_overflow_chain() {
        let mut t = Tree::new(NodeKind::Dir, "/");
        let root = t.root();
        for _ in 0..3 {
            t.add_child(root, NodeKind::File, "x", u64::MAX, u64::MAX)
                .unwrap();
        }
        t.aggregate();
        assert!(t.size_overflow());
        assert_eq!(t.get(root).unwrap().size, u64::MAX);
    }

    #[test]
    fn test_sec_uc002_path_on_unknown_id_does_not_panic() {
        let t = Tree::new(NodeKind::Dir, "/");
        let p = t.path(NodeId(99));
        assert!(p.is_empty());
    }

    #[test]
    fn test_sec_uc002_invalid_parent_does_not_corrupt_state() {
        let mut t = Tree::new(NodeKind::Dir, "/");
        let _ = t.add_child(NodeId(5), NodeKind::File, "x", 0, 0);
        assert_eq!(t.len(), 1);
        let real = t.add_child(t.root(), NodeKind::File, "ok", 1, 1).unwrap();
        assert_eq!(real.index(), 1);
    }
}
