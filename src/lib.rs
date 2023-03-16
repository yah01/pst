use std::cell::RefCell;
use std::cmp::min;
use std::rc::Rc;

// Indicates how the parent combines the values of children
trait Add<T> {
    fn add(a: Option<T>, b: Option<T>) -> Option<T> {
        None
    }
}

// Indicates how the parent removes the values of children before query time range
trait Subtract<T> {
    fn subtract(a: Option<T>, b: Option<T>) -> Option<T> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct Pst<T: Clone> {
    // Value range
    left: usize,
    right: usize,

    versions: Vec<NodePtr<T>>,
}

impl<T: Clone> Pst<T> {
    pub fn new(left: usize, right: usize) -> Self {
        Self {
            left,
            right,
            versions: vec![Node::new()],
        }
    }

    pub fn insert(&mut self, pos: usize, value: T) {
        let last = self.versions.last().cloned();
        self.versions.push(Node::new());
        let root = self.versions.last().cloned();

        Self::insert_impl(pos, self.left, self.right, root, last, value);
    }

    // node must be not None
    fn insert_impl(
        pos: usize,
        left: usize,
        right: usize,
        node: Option<NodePtr<T>>,
        last: Option<NodePtr<T>>,
        value: T,
    ) {
        let binding = node.unwrap().clone();
        let mut node = binding.borrow_mut();
        if right - left <= 1 {
            node.value = Some(value);
            return;
        }

        let last_left = last.as_ref().map_or(None, |v| v.borrow().left.clone());
        let last_right = last.as_ref().map_or(None, |v| v.borrow().right.clone());
        let mid = (left + right) / 2;
        if pos < mid {
            node.left = Some(Node::new());
            node.right = last_right;
            Self::insert_impl(pos, left, mid, node.left.clone(), last_left, value);
        } else {
            node.left = last_left;
            node.right = Some(Node::new());
            Self::insert_impl(pos, mid, right, node.right.clone(), last_right, value)
        }

        node.add_up()
    }

    pub fn query(&self, pos: usize, version: usize) -> Option<T> {
        let version = min(version, self.versions.len() - 1);
        let mut node = Some(self.versions[version].clone());

        let mut left = self.left;
        let mut right = self.right;
        while right - left > 1 && node.is_some() {
            let inner = node.unwrap();
            let mid = (left + right) / 2;
            if pos < mid {
                right = mid;
                node = inner.borrow().left.clone();
            } else {
                left = mid;
                node = inner.borrow().right.clone();
            }
        }

        node.map_or(None, |v| v.borrow().value.clone())
    }
}

type NodePtr<T> = Rc<RefCell<Node<T>>>;

#[derive(Debug, Clone)]
struct Node<T: Clone> {
    left: Option<NodePtr<T>>,
    right: Option<NodePtr<T>>,
    value: Option<T>,
}

impl<T: Clone> Node<T> {
    pub fn new() -> NodePtr<T> {
        Rc::new(RefCell::new(Self {
            left: None,
            right: None,
            value: None,
        }))
    }

    pub(crate) fn add_up(&mut self) {
        self.value = None;
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn normal() {
        let n = 8;
        let mut pst = Pst::new(0, n);
        println!("test normal");
        for i in 0..n {
            pst.insert(i, i + 1);
        }

        for version in 0..n + 1 {
            println!("test on version {}", version);
            for i in 0..n {
                let v = pst.query(i, version);
                if version <= i {
                    assert_eq!(v, None);
                } else {
                    assert_eq!(v, Some(i + 1), "version={} i={}", version, i);
                }
            }
        }
    }
}
