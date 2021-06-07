use std::iter::Once;
use std::iter::once;

/// Given a result of iterators, return an iterator of results.
///
/// If the result is `Ok(iter)`, this returns
/// an iterator that yields `Ok(item)` for each `item` in `iter`.
/// If the result is `Err(err)`, this returns
/// an iterator that yields `Err(err)` once.
pub fn iter_result_iter<I, E>(result: Result<I, E>) -> IterResultIter<I, E>
{
    let inner = result.map_err(once);
    IterResultIter{inner}
}

/// Iterator returned by [`iter_result_iter`].
pub struct IterResultIter<I, E>
{
    inner: Result<I, Once<E>>,
}

impl<I, E> Iterator for IterResultIter<I, E>
    where I: Iterator
{
    type Item = Result<I::Item, E>;

    fn next(&mut self) -> Option<Self::Item>
    {
        match &mut self.inner {
            Ok(iter)  => iter.next().map(Ok),
            Err(iter) => iter.next().map(Err),
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_iter_result_iter()
    {
        use std::slice::Iter;

        let r1: Result<_, ()> = Ok([1, 2, 3].iter());
        let r2: Result<Iter<'_, i32>, _> = Err(1);

        assert_eq!(
            iter_result_iter(r1).collect::<Vec<_>>(),
            vec![Ok(&1), Ok(&2), Ok(&3)],
        );

        assert_eq!(
            iter_result_iter(r2).collect::<Vec<_>>(),
            vec![Err(1)],
        );
    }
}
