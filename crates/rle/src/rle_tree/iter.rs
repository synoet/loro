use crate::Rle;

use super::{
    node::LeafNode,
    tree_trait::{Position, RleTreeTrait},
    SafeCursor, SafeCursorMut, UnsafeCursor,
};

/// cursor's and `end_cursor`'s length means nothing in this context
pub struct Iter<'some, 'bump, T: Rle, A: RleTreeTrait<T>> {
    cursor: Option<UnsafeCursor<'some, 'bump, T, A>>,
    end_cursor: Option<UnsafeCursor<'some, 'bump, T, A>>,
}

pub struct IterMut<'some, 'bump, T: Rle, A: RleTreeTrait<T>> {
    cursor: Option<UnsafeCursor<'some, 'bump, T, A>>,
    end_cursor: Option<UnsafeCursor<'some, 'bump, T, A>>,
}

impl<'tree, 'bump: 'tree, T: Rle, A: RleTreeTrait<T>> IterMut<'tree, 'bump, T, A> {
    #[inline]
    pub fn new(node: Option<&'tree mut LeafNode<'bump, T, A>>) -> Self {
        if node.is_none() {
            return Self {
                cursor: None,
                end_cursor: None,
            };
        }

        let node = node.unwrap();
        Self {
            cursor: Some(UnsafeCursor::new(node.into(), 0, 0, Position::Start, 0)),
            end_cursor: None,
        }
    }

    #[inline]
    pub fn from_cursor(
        mut start: SafeCursorMut<'tree, 'bump, T, A>,
        mut end: Option<SafeCursor<'tree, 'bump, T, A>>,
    ) -> Option<Self> {
        if start.0.pos == Position::After {
            start = start.next_elem_start()?
        }

        if let Some(end_inner) = end {
            if end_inner.0.pos == Position::End || end_inner.0.pos == Position::After {
                end = end_inner.next_elem_start();
            }
        }

        Some(Self {
            cursor: Some(UnsafeCursor::new(
                start.0.leaf,
                start.0.index,
                start.0.offset,
                start.0.pos,
                0,
            )),
            end_cursor: end.map(|x| UnsafeCursor::new(x.0.leaf, x.0.index, x.0.offset, x.0.pos, 0)),
        })
    }
}

impl<'tree, 'bump: 'tree, T: Rle, A: RleTreeTrait<T>> Iter<'tree, 'bump, T, A> {
    #[inline]
    pub fn new(node: Option<&'tree LeafNode<'bump, T, A>>) -> Self {
        if node.is_none() {
            return Self {
                cursor: None,
                end_cursor: None,
            };
        }

        let node = node.unwrap();
        Self {
            cursor: Some(UnsafeCursor::new(node.into(), 0, 0, Position::Start, 0)),
            end_cursor: None,
        }
    }

    #[inline]
    pub fn from_cursor(
        mut start: SafeCursor<'tree, 'bump, T, A>,
        mut end: Option<SafeCursor<'tree, 'bump, T, A>>,
    ) -> Option<Self> {
        if start.0.pos == Position::After {
            start = start.next_elem_start()?
        }

        if let Some(end_inner) = end {
            if end_inner.0.pos == Position::End || end_inner.0.pos == Position::After {
                end = end_inner.next_elem_start();
            }
        }

        Some(Self {
            cursor: Some(start.0),
            end_cursor: end.map(|x| x.0),
        })
    }
}

impl<'tree, 'bump: 'tree, T: Rle, A: RleTreeTrait<T>> Iterator for Iter<'tree, 'bump, T, A> {
    type Item = SafeCursor<'tree, 'bump, T, A>;

    fn next(&mut self) -> Option<Self::Item> {
        self.cursor?;
        if self.end_cursor.is_some() {
            let start = self.cursor.as_mut().unwrap();
            let end = self.end_cursor.as_mut().unwrap();
            if start.leaf == end.leaf && start.index == end.index && start.offset == end.offset {
                return None;
            }
        }

        while let Some(ref mut cursor) = self.cursor {
            // SAFETY: we are sure that the cursor is valid
            let node = unsafe { cursor.leaf.as_ref() };
            match node.children.get(cursor.index) {
                Some(_) => {
                    if let Some(end) = self.end_cursor {
                        if cursor.leaf == end.leaf && end.index == cursor.index {
                            if cursor.offset == end.offset {
                                return None;
                            } else {
                                // SAFETY: we just checked that the child exists
                                let ans = Some(unsafe {
                                    SafeCursor::new(
                                        node.into(),
                                        cursor.index,
                                        cursor.offset,
                                        Position::from_offset(
                                            cursor.offset as isize,
                                            node.children[cursor.index].len(),
                                        ),
                                        end.offset - cursor.offset,
                                    )
                                });
                                self.cursor = None;
                                return ans;
                            }
                        }
                    }

                    let child_len = node.children[cursor.index].len();
                    // SAFETY: we just checked that the child exists
                    let ans = Some(unsafe {
                        SafeCursor::new(
                            node.into(),
                            cursor.index,
                            cursor.offset,
                            Position::from_offset(cursor.offset as isize, child_len),
                            child_len - cursor.offset,
                        )
                    });

                    cursor.index += 1;
                    cursor.offset = 0;
                    return ans;
                }
                None => match node.next() {
                    Some(next) => {
                        cursor.leaf = next.into();
                        cursor.index = 0;
                        cursor.offset = 0;
                        continue;
                    }
                    None => return None,
                },
            }
        }

        None
    }
}

impl<'tree, 'bump: 'tree, T: Rle, A: RleTreeTrait<T>> Iterator for IterMut<'tree, 'bump, T, A> {
    type Item = SafeCursorMut<'tree, 'bump, T, A>;

    fn next(&mut self) -> Option<Self::Item> {
        self.cursor?;
        if self.end_cursor.is_some() {
            let start = self.cursor.as_mut().unwrap();
            let end = self.end_cursor.as_mut().unwrap();
            if start.leaf == end.leaf && start.index == end.index && start.offset == end.offset {
                return None;
            }
        }

        while let Some(ref mut cursor) = self.cursor {
            // SAFETY: we are sure that the cursor is valid
            let node = unsafe { cursor.leaf.as_ref() };
            match node.children.get(cursor.index) {
                Some(_) => {
                    if let Some(end) = self.end_cursor {
                        if cursor.leaf == end.leaf && end.index == cursor.index {
                            if cursor.offset == end.offset {
                                return None;
                            } else {
                                // SAFETY: we just checked that the child exists
                                let ans = Some(unsafe {
                                    SafeCursorMut::new(
                                        node.into(),
                                        cursor.index,
                                        cursor.offset,
                                        Position::from_offset(
                                            cursor.offset as isize,
                                            node.children[cursor.index].len(),
                                        ),
                                        end.offset - cursor.offset,
                                    )
                                });
                                cursor.offset = end.offset;
                                self.cursor = None;
                                return ans;
                            }
                        }
                    }

                    let child_len = node.children[cursor.index].len();
                    // SAFETY: we just checked that the child exists
                    let ans = Some(unsafe {
                        SafeCursorMut::new(
                            node.into(),
                            cursor.index,
                            cursor.offset,
                            Position::from_offset(cursor.offset as isize, child_len),
                            child_len - cursor.offset,
                        )
                    });

                    cursor.index += 1;
                    cursor.offset = 0;
                    return ans;
                }
                None => match node.next() {
                    Some(next) => {
                        cursor.leaf = next.into();
                        cursor.index = 0;
                        cursor.offset = 0;
                        continue;
                    }
                    None => return None,
                },
            }
        }

        None
    }
}
