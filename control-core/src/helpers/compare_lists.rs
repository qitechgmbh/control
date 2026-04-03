pub struct ListChange<'t, T>
where
    T: PartialEq,
{
    pub added: Vec<&'t T>,
    pub removed: Vec<&'t T>,
    pub same: Vec<&'t T>,
}

#[must_use]
pub fn compare_lists<'oldlist, 'newlist, 't, T: PartialEq>(
    old_list: &'oldlist Vec<T>,
    new_list: &'newlist Vec<T>,
) -> ListChange<'t, T>
where
    T: PartialEq,
    'oldlist: 't,
    'newlist: 't,
{
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut same = Vec::new();

    for item in new_list {
        if old_list.iter().any(|x| x == item) {
            same.push(item);
        } else {
            added.push(item);
        }
    }

    for item in old_list {
        if !new_list.iter().any(|x| x == item) {
            removed.push(item);
        }
    }

    ListChange {
        added,
        removed,
        same,
    }
}
