/// Represents the type for ids
pub type Id = usize;

/// Represents the id that is not used for allocation but is instead reserved
/// by applications for grabbing a different, unique id
pub const EPHEMERAL_ID: Id = 0;

/// Represents the allocator of unique ids
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct IdAllocator {
    /// Represents a counter that keeps track of where an allocator is when
    /// consuming a new id
    next_id: Option<Id>,

    /// Represents ids that have been freed and can be reused
    freed: Vec<Id>,
}

impl IdAllocator {
    /// Creates a new allocator using the default parameters
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates the next id in the allocator to the id provided.
    #[inline]
    pub fn set_next_id(&mut self, id: Id) {
        self.next_id = Some(id);
    }

    /// Adjusts the next id of the allocator to beyond the given id if possible.
    /// This is useful when ids are allocated outside of the allocator and we
    /// want to make sure that the allocator does not duplicate those ids.
    #[inline]
    pub fn mark_external_id(&mut self, id: Id) {
        if let Some(nid) = self.next_id {
            let is_less = nid <= id;
            if is_less && id == Id::MAX {
                self.next_id = None;
            } else if is_less {
                self.next_id = Some(id + 1);
            }
        }
    }

    /// Indicates whether or not the allocator has more ids available, either
    /// from the freed pool or by incrementing its counter
    #[inline]
    pub fn has_next(&self) -> bool {
        self.next_id.is_some() || !self.freed.is_empty()
    }

    /// Represents the ids that have been freed and will be returned by
    /// the allocator before generating new ids
    #[inline]
    pub fn freed(&self) -> &[Id] {
        &self.freed
    }
}

impl Default for IdAllocator {
    /// Creates a new allocator starting with no freed ids and beginning
    /// with the next id after the ephemeral id
    fn default() -> Self {
        Self {
            next_id: Some(EPHEMERAL_ID + 1),
            freed: Vec::new(),
        }
    }
}

impl Iterator for IdAllocator {
    type Item = Id;

    /// Produces a range by either yielding one of the freed pool ranges
    /// or allocating a new range of ids
    ///
    /// This should always yield a new range and should be assumed to be
    /// infinite. If the allocator has run out of ids, it will return None,
    /// and this would indicate a problem that should panic.
    fn next(&mut self) -> Option<Self::Item> {
        // If we have some id available where we do not need to allocate
        // a new id, return that instead of a new allocation
        if let Some(id) = self.freed.pop() {
            Some(id)

        // We still have an id available, so we consume it
        } else if let Some(id) = self.next_id {
            // If this is the last id, then we update our next id to none
            if id == Id::MAX {
                self.next_id = None;

            // Otherwise, we can still increment our counter and should do so
            } else {
                self.next_id = Some(id + 1);
            }

            Some(id)

        // If we have reached the maximum possible ids for this allocator,
        // we should indicate there are no more avaialble.
        } else {
            None
        }
    }
}

impl Extend<Id> for IdAllocator {
    /// Extends the collection of freed ids with the given iterator
    /// of ids
    fn extend<I: IntoIterator<Item = Id>>(&mut self, iter: I) {
        self.freed.extend(iter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_should_return_next_id_if_no_freed_available_and_not_reached_limit() {
        let mut id_alloc = IdAllocator::new();

        assert_eq!(id_alloc.next(), Some(1));
        assert_eq!(id_alloc.next(), Some(2));
        assert_eq!(id_alloc.next(), Some(3));
    }

    #[test]
    fn next_should_return_freed_id_if_available() {
        let mut id_alloc = IdAllocator::new();
        id_alloc.extend(vec![9, 8, 7]);

        assert_eq!(id_alloc.next(), Some(7));
        assert_eq!(id_alloc.next(), Some(8));
        assert_eq!(id_alloc.next(), Some(9));

        assert_eq!(id_alloc.next(), Some(1));
        assert_eq!(id_alloc.next(), Some(2));
        assert_eq!(id_alloc.next(), Some(3));
    }

    #[test]
    fn next_should_return_none_if_no_freed_and_reached_limit() {
        let mut id_alloc = IdAllocator::new();
        id_alloc.set_next_id(Id::MAX);

        assert_eq!(id_alloc.next(), Some(Id::MAX));
        assert_eq!(id_alloc.next(), None);
    }

    #[test]
    fn extend_should_append_ids_to_freed_list() {
        let mut id_alloc = IdAllocator::new();
        id_alloc.extend(vec![9, 8, 7]);
        id_alloc.extend(vec![6, 5, 4]);

        assert_eq!(id_alloc.freed(), &[9, 8, 7, 6, 5, 4]);
    }

    #[test]
    fn mark_external_id_should_move_next_id_beyond_provided_id_if_less_than_id() {
        let mut id_alloc = IdAllocator::new();

        id_alloc.set_next_id(3);
        id_alloc.mark_external_id(999);
        assert_eq!(id_alloc.next(), Some(1000));
    }

    #[test]
    fn mark_external_id_should_move_next_id_beyond_provided_id_if_equal_to_id() {
        let mut id_alloc = IdAllocator::new();

        id_alloc.set_next_id(999);
        id_alloc.mark_external_id(999);
        assert_eq!(id_alloc.next(), Some(1000));
    }

    #[test]
    fn mark_external_id_should_not_move_next_id_if_given_id_is_less_than_it() {
        let mut id_alloc = IdAllocator::new();
        id_alloc.set_next_id(1000);
        id_alloc.mark_external_id(1);
        assert_eq!(id_alloc.next(), Some(1000));
    }

    #[test]
    fn mark_external_id_should_not_move_next_id_if_next_id_is_already_none() {
        let mut id_alloc = IdAllocator::new();
        id_alloc.next_id = None;
        id_alloc.mark_external_id(1);
        assert_eq!(id_alloc.next(), None);
    }
}
