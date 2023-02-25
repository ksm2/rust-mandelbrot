use rayon::iter::plumbing::{bridge, Consumer, Producer, ProducerCallback, UnindexedConsumer};
use rayon::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Dim {
    pub width: i32,
    pub height: i32,
}

impl Dim {
    pub fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    pub fn len(&self) -> usize {
        self.width as usize * self.height as usize
    }
}

impl<'a> IntoIterator for &'a Dim {
    type Item = (i32, i32);
    type IntoIter = DimIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        DimIterator::new(self)
    }
}

impl<'a> IntoParallelIterator for &'a Dim {
    type Iter = DimParallelIterator<'a>;
    type Item = (i32, i32);

    fn into_par_iter(self) -> Self::Iter {
        DimParallelIterator { dim: self }
    }
}

pub struct DimParallelIterator<'a> {
    dim: &'a Dim,
}

impl<'a> ParallelIterator for DimParallelIterator<'a> {
    type Item = (i32, i32);

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn opt_len(&self) -> Option<usize> {
        Some(self.dim.len())
    }
}

impl<'a> IndexedParallelIterator for DimParallelIterator<'a> {
    fn len(&self) -> usize {
        self.dim.len()
    }

    fn drive<C: Consumer<Self::Item>>(self, consumer: C) -> C::Result {
        bridge(self, consumer)
    }

    fn with_producer<CB: ProducerCallback<Self::Item>>(self, callback: CB) -> CB::Output {
        let producer = DimProducer::from(self);
        callback.callback(producer)
    }
}

struct DimProducer<'a> {
    from: usize,
    to: usize,
    dim: &'a Dim,
}

impl<'a> DimProducer<'a> {
    fn new(from: usize, to: usize, dim: &'a Dim) -> Self {
        Self { from, to, dim }
    }
}

impl<'a> Producer for DimProducer<'a> {
    type Item = (i32, i32);
    type IntoIter = DimIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        DimIterator::with_bounds(self.dim, self.from, self.to)
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        (
            Self::new(self.from, self.from + index, self.dim),
            Self::new(self.from + index, self.to, self.dim),
        )
    }
}

impl<'a> From<DimParallelIterator<'a>> for DimProducer<'a> {
    fn from(value: DimParallelIterator<'a>) -> Self {
        Self::new(0, value.dim.len(), value.dim)
    }
}

pub struct DimIterator<'a> {
    dim: &'a Dim,
    offset: usize,
    offset_back: usize,
}

impl<'a> DimIterator<'a> {
    fn new(dim: &'a Dim) -> Self {
        Self::with_bounds(dim, 0, dim.len())
    }

    fn with_bounds(dim: &'a Dim, from: usize, to: usize) -> Self {
        Self {
            dim,
            offset: from,
            offset_back: to,
        }
    }
}

impl<'a> Iterator for DimIterator<'a> {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.offset_back {
            return None;
        }

        let index = self.offset;
        let x = index / self.dim.height as usize;
        let y = index % self.dim.height as usize;
        self.offset += 1;
        Some((x as i32, y as i32))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.offset_back - self.offset;
        (size, Some(size))
    }
}

impl<'a> DoubleEndedIterator for DimIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.offset_back == 0 {
            return None;
        }

        self.offset_back -= 1;
        if self.offset_back < self.offset {
            return None;
        }

        let index = self.offset_back;
        let x = index / self.dim.height as usize;
        let y = index % self.dim.height as usize;
        Some((x as i32, y as i32))
    }
}

impl<'a> ExactSizeIterator for DimIterator<'a> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iterates_over_empty() {
        let dim = Dim::new(0, 0);
        assert_eq!(0, dim.len());

        let producer = DimProducer::from(dim.into_par_iter());
        let mut iter = producer.into_iter();
        assert_eq!(0, iter.offset);
        assert_eq!(0, iter.offset_back);
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next_back());

        let vec = dim.into_par_iter().collect::<Vec<(i32, i32)>>();
        assert_eq!(Vec::<(i32, i32)>::new(), vec);
    }

    #[test]
    fn iterates_over_non_empty() {
        let dim = Dim::new(2, 3);
        assert_eq!(6, dim.len());

        let producer = DimProducer::from(dim.into_par_iter());
        let mut iter = producer.into_iter();
        assert_eq!(0, iter.offset);
        assert_eq!(6, iter.offset_back);
        assert_eq!((6, Some(6)), iter.size_hint());

        assert_eq!(Some((0, 0)), iter.next());
        assert_eq!((5, Some(5)), iter.size_hint());

        assert_eq!(Some((0, 1)), iter.next());
        assert_eq!((4, Some(4)), iter.size_hint());

        assert_eq!(Some((0, 2)), iter.next());
        assert_eq!((3, Some(3)), iter.size_hint());

        assert_eq!(Some((1, 0)), iter.next());
        assert_eq!((2, Some(2)), iter.size_hint());

        assert_eq!(Some((1, 2)), iter.next_back());
        assert_eq!((1, Some(1)), iter.size_hint());

        assert_eq!(Some((1, 1)), iter.next_back());
        assert_eq!((0, Some(0)), iter.size_hint());
        assert_eq!(None, iter.next_back());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn it_collects() {
        let dim = Dim::new(2, 3);

        let vec = dim.into_par_iter().collect::<Vec<(i32, i32)>>();
        assert_eq!(vec![(0, 0), (0, 1), (0, 2), (1, 0), (1, 1), (1, 2)], vec);
    }

    #[test]
    fn producer_can_be_split() {
        let dim = Dim::new(2, 3);
        assert_eq!(6, dim.len());

        let producer = DimProducer::from(dim.into_par_iter());
        let (prod1, prod2) = producer.split_at(2);
        assert_eq!(0, prod1.from);
        assert_eq!(2, prod1.to);
        assert_eq!(2, prod2.from);
        assert_eq!(6, prod2.to);

        let vec1 = prod1.into_iter().collect::<Vec<_>>();
        let vec2 = prod2.into_iter().collect::<Vec<_>>();
        assert_eq!(vec![(0, 0), (0, 1)], vec1);
        assert_eq!(vec![(0, 2), (1, 0), (1, 1), (1, 2)], vec2);
    }

    #[test]
    fn producer_can_be_split_at_0() {
        let dim = Dim::new(2, 3);
        assert_eq!(6, dim.len());

        let producer = DimProducer::from(dim.into_par_iter());
        let (prod1, prod2) = producer.split_at(0);
        assert_eq!(0, prod1.from);
        assert_eq!(0, prod1.to);
        assert_eq!(0, prod2.from);
        assert_eq!(6, prod2.to);

        let vec1 = prod1.into_iter().collect::<Vec<_>>();
        let vec2 = prod2.into_iter().collect::<Vec<_>>();
        assert_eq!(Vec::<(i32, i32)>::new(), vec1);
        assert_eq!(vec![(0, 0), (0, 1), (0, 2), (1, 0), (1, 1), (1, 2)], vec2);
    }

    #[test]
    fn producer_can_be_split_at_n() {
        let dim = Dim::new(2, 3);
        assert_eq!(6, dim.len());

        let producer = DimProducer::from(dim.into_par_iter());
        let (prod1, prod2) = producer.split_at(6);
        assert_eq!(0, prod1.from);
        assert_eq!(6, prod1.to);
        assert_eq!(6, prod2.from);
        assert_eq!(6, prod2.to);

        let vec1 = prod1.into_iter().collect::<Vec<_>>();
        let vec2 = prod2.into_iter().collect::<Vec<_>>();
        assert_eq!(vec![(0, 0), (0, 1), (0, 2), (1, 0), (1, 1), (1, 2)], vec1);
        assert_eq!(Vec::<(i32, i32)>::new(), vec2);
    }
}
