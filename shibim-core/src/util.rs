/*adapted from itertools */
pub trait Itertools : Iterator{
    fn partition_result<A, B, F, L, R>(self, mut predicate: F) -> (A, B)
            where Self: Sized,
                F: FnMut(Self::Item) -> Result<L, R>,
                A: Default + Extend<L>,
                B: Default + Extend<R>,
        {
            let mut left = A::default();
            let mut right = B::default();

            self.for_each(|val| match predicate(val) {
                Ok(v) => left.extend(Some(v)),
                Err(v) => right.extend(Some(v)),
            });

            (left, right)
    }
}

impl<I: Iterator> Itertools for I {}