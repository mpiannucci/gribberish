
pub struct ScaleGribValue<I> {
    iter: I, 
    binary_scale_factor: f64,
    decimal_scale_factor: f64,
    reference_value: f64,
}

impl<I> ScaleGribValue<I> {
    pub fn new(iter: I, binary_scale_factor: i16, decimal_scale_factor: i16, reference_value: f32) -> Self {
        Self { 
            iter, 
            binary_scale_factor: 2_f64.powi(binary_scale_factor as i32),
            decimal_scale_factor: 10_f64.powi(-decimal_scale_factor as i32), 
            reference_value: reference_value as f64,
        }
    }
}

impl<I, T> Iterator for ScaleGribValue<I>
where
    I: Iterator<Item = T>,
    T: num::ToPrimitive + Copy,
{
    type Item = f64;
    fn next(&mut self) -> Option<Self::Item> {
        let v = self.iter.next()?.to_f64()?;
        Some((v * self.binary_scale_factor + self.reference_value) * self.decimal_scale_factor)
    }
}

pub trait ScaleGribValueIterator<T>: Iterator<Item = T> + Sized {
    fn scale_value_by(self, binary_scale_factor: i16, decimal_scale_factor: i16, reference_value: f32) -> ScaleGribValue<Self> {
        ScaleGribValue::new(self, binary_scale_factor, decimal_scale_factor, reference_value)
    }
}

impl<T, I: Iterator<Item = T>> ScaleGribValueIterator<T> for I {}