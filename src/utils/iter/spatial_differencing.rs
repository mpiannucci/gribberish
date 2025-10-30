
pub struct FirstOrderSpatialDifferencing<I> {
    iter: I,
    d1: i32,
    dmin: i32,
    prev: i32,
    current_index: usize,
}

impl <I> FirstOrderSpatialDifferencing<I> {
    pub fn new(iter: I, d1: i32, dmin: i32) -> Self {
        Self {
            iter,
            d1,
            dmin,
            prev: d1,
            current_index: 0,
        }
    }
}

impl <I> Iterator for FirstOrderSpatialDifferencing<I> where I: Iterator<Item = i32> {
    type Item = i32;
    fn next(&mut self) -> Option<Self::Item> {
        let current_index = self.current_index;
        self.current_index += 1;

        if current_index == 0 {
            _ = self.iter.next()?;
            Some(self.d1)
        } else {
            let next_value = self.iter.next()?;
            let next_value = next_value + self.prev + self.dmin;
            self.prev = next_value;
            Some(next_value)
        }
    }
}


pub struct SecondOrderSpatialDifferencing<I> {
    iter: I,
    d1: i32,
    d2: i32,
    dmin: i32,
    prev: i32,
    prev2: i32,
    current_index: usize,
}

impl <I> SecondOrderSpatialDifferencing<I> {
    pub fn new(iter: I, d1: i32, d2: i32, dmin: i32) -> Self {
        Self {
            iter,
            d1,
            d2,
            dmin,
            prev: d2,
            prev2: d1,
            current_index: 0,
        }
    }
}

impl <I> Iterator for SecondOrderSpatialDifferencing<I> where I: Iterator<Item = i32> {
    type Item = i32;
    fn next(&mut self) -> Option<Self::Item> {
        let current_index = self.current_index;
        self.current_index += 1;

        match current_index {
            0 => {
                _ = self.iter.next()?;
                Some(self.d1)
            }, 
            1 => {
                _ = self.iter.next()?;
                Some(self.d2)
            },
            _ => {
                let next_value = self.iter.next()?;
                let next_value = next_value + 2 * self.prev - self.prev2 + self.dmin;
                self.prev2 = self.prev;
                self.prev = next_value;
                Some(next_value)
            }
        }
    }
}

pub trait SpatialDifferencingIterator<T>: Iterator<Item = T> + Sized {
    fn apply_first_order_spatial_differencing(self, d1: i32, dmin: i32) -> FirstOrderSpatialDifferencing<Self> {
        FirstOrderSpatialDifferencing::new(self, d1, dmin)
    }

    fn apply_second_order_spatial_differencing(self, d1: i32, d2: i32, dmin: i32) -> SecondOrderSpatialDifferencing<Self> {
        SecondOrderSpatialDifferencing::new(self, d1, d2, dmin)
    }
}

impl<T, I: Iterator<Item = T>> SpatialDifferencingIterator<T> for I {}