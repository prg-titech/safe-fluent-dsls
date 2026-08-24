use tree_sitter::Node;
use tree_sitter::TreeCursor;

pub struct TreeIterator<'a> {
    cursor: TreeCursor<'a>,
    was_initialized: bool,
}

impl<'a> TreeIterator<'a> {
    pub fn new(cursor: TreeCursor<'a>) -> Self {
        TreeIterator {
            cursor,
            was_initialized: false,
        }
    }
}

impl<'a> Iterator for TreeIterator<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.was_initialized {
            if self.cursor.goto_first_child() {
                return Some(self.cursor.node());
            }
            while !self.cursor.goto_next_sibling() {
                if !self.cursor.goto_parent() {
                    return None;
                }
            }
            return Some(self.cursor.node());
        } else {
            self.was_initialized = true;
            Some(self.cursor.node())
        }
    }
}
