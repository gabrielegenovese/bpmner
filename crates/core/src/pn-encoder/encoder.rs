
/// Returns P∅(X): all non-empty subsets of X, as used in the OR-split /
/// OR-join encodings (Table 1), where a transition t_S is generated for
/// every S ∈ P∅(E). `X` is taken as a slice to fix an iteration order;
/// the returned subsets preserve the relative order of `X`'s elements.
pub fn non_empty_subsets<T: Clone + Ord>(x: &[T]) -> Vec<Vec<T>> {
    let n = x.len();
    assert!(n <= 63, "non_empty_subsets: input too large for bitmask enumeration");
    let mut result = Vec::with_capacity((1usize << n).saturating_sub(1));
    for mask in 1..(1u64 << n) {
        let mut subset = Vec::new();
        for (i, item) in x.iter().enumerate() {
            if mask & (1 << i) != 0 {
                subset.push(item.clone());
            }
        }
        result.push(subset);
    }
    result
}

/// Canonical string label for a subset, used to build transition/place
/// names like `t_{e8,e9}` in the OR-gateway encoding. Elements are
/// sorted and de-duplicated for a stable, order-independent name.
pub fn subset_label<T: AsRef<str>>(subset: &[T]) -> String {
    let mut items: Vec<&str> = subset.iter().map(|s| s.as_ref()).collect();
    items.sort_unstable();
    items.dedup();
    items.join(",")
}
