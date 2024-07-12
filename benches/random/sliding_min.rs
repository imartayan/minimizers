use std::collections::VecDeque;

use super::Minimizer;

pub trait SlidingMin<V> {
    /// Initialize a new datastructure with window size `w`.
    fn new(w: usize) -> Self;
    /// Push a new value, starting at position 0.
    /// Return the pos and value of the minimum of the last w elements.
    fn push(&mut self, val: V) -> Elem<V>;
}

#[derive(Clone, Copy)]
pub struct Elem<Val> {
    pub pos: usize,
    pub val: Val,
}

pub struct MonotoneQueue<Val: Ord> {
    w: usize,
    pos: usize,
    /// A queue of (pos, val) objects.
    /// Both pos and val values are always increasing, so that the smallest
    /// value is always at the front.
    q: VecDeque<Elem<Val>>,
}

impl<Val: Ord + Copy> SlidingMin<Val> for MonotoneQueue<Val> {
    fn new(w: usize) -> Self {
        assert!(w > 0);
        Self {
            w,
            pos: 0,
            q: VecDeque::new(),
        }
    }

    fn push(&mut self, val: Val) -> Elem<Val> {
        // Strictly larger preceding `k` are removed, so that the queue remains
        // non-decreasing.
        while let Some(back) = self.q.back() {
            if back.val > val {
                self.q.pop_back();
            } else {
                break;
            }
        }
        self.q.push_back(Elem { pos: self.pos, val });
        let front = self.q.front().unwrap(); // Safe, because we just pushed.
        if self.pos - front.pos >= self.w {
            self.q.pop_front();
        }
        self.pos += 1;
        *self.q.front().unwrap() // Safe, because w > 0.
    }
}

pub struct V3Queue {
    pub w: usize,
    pub k: usize,
}

impl Minimizer for V3Queue {
    fn minimizers(&self, text: &[u8]) -> Vec<usize> {
        let mut q = MonotoneQueue::new(self.w);
        let mut kmers = text.windows(self.k);
        // Inset the first w-1 k-mers, that do not yet form a full window.
        for kmer in kmers.by_ref().take(self.w - 1) {
            q.push(fxhash::hash(kmer));
        }
        kmers.map(|kmer| q.push(fxhash::hash(kmer)).pos).collect()
    }
}
