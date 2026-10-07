//! From the encoder's token states to one unit vector per text.

use candle_core::{D, DType, Result, Tensor};

/// State of the first token (`[CLS]`): `(batch, seq, hidden)` → `(batch, hidden)`.
pub fn cls(hidden: &Tensor) -> Result<Tensor> {
    hidden.narrow(1, 0, 1)?.squeeze(1)
}

/// Mean of the token states the attention mask keeps, so padding of a batch never shifts a
/// text's vector: `(batch, seq, hidden)` and `(batch, seq)` → `(batch, hidden)`.
pub fn mean(hidden: &Tensor, mask: &Tensor) -> Result<Tensor> {
    let mask = mask.to_dtype(hidden.dtype())?.unsqueeze(2)?;
    let summed = hidden.broadcast_mul(&mask)?.sum(1)?;
    // A text of only padding cannot come from the tokenizer (it adds special tokens), but a zero
    // count must not turn into NaN either.
    let counts = mask.sum(1)?.clamp(1e-9, f64::MAX)?;
    summed.broadcast_div(&counts)
}

/// Rows scaled to unit length; a zero row stays zero.
pub fn l2_normalize(x: &Tensor) -> Result<Tensor> {
    let norm = x
        .to_dtype(DType::F32)?
        .sqr()?
        .sum_keepdim(D::Minus1)?
        .sqrt()?
        .clamp(1e-12, f64::MAX)?;
    x.broadcast_div(&norm)
}

#[cfg(test)]
mod tests {
    use candle_core::Device;

    use super::*;

    fn tensor3<const B: usize>(rows: [[[f32; 2]; 3]; B]) -> Tensor {
        Tensor::new(&rows, &Device::Cpu).unwrap()
    }

    #[test]
    fn cls_takes_the_first_token() {
        let h = tensor3([
            [[1., 2.], [3., 4.], [5., 6.]],
            [[7., 8.], [9., 0.], [0., 0.]],
        ]);
        let v: Vec<Vec<f32>> = cls(&h).unwrap().to_vec2().unwrap();
        assert_eq!(v, [[1., 2.], [7., 8.]]);
    }

    #[test]
    fn mean_ignores_padding() {
        let h = tensor3([
            [[1., 2.], [3., 4.], [100., 100.]],
            [[2., 2.], [4., 6.], [6., 10.]],
        ]);
        let mask = Tensor::new(&[[1u32, 1, 0], [1, 1, 1]], &Device::Cpu).unwrap();
        let v: Vec<Vec<f32>> = mean(&h, &mask).unwrap().to_vec2().unwrap();
        assert_eq!(v, [[2., 3.], [4., 6.]]);
    }

    #[test]
    fn mean_of_an_all_padding_row_is_zero_not_nan() {
        let h = tensor3([[[1., 2.], [3., 4.], [5., 6.]]]);
        let mask = Tensor::new(&[[0u32, 0, 0]], &Device::Cpu).unwrap();
        let v: Vec<Vec<f32>> = mean(&h, &mask).unwrap().to_vec2().unwrap();
        assert_eq!(v, [[0., 0.]]);
    }

    #[test]
    fn normalize_gives_unit_rows_and_keeps_zero() {
        let x = Tensor::new(&[[3f32, 4.], [0., 0.], [-2., 0.]], &Device::Cpu).unwrap();
        let v: Vec<Vec<f32>> = l2_normalize(&x).unwrap().to_vec2().unwrap();
        assert_eq!(v, [[0.6, 0.8], [0., 0.], [-1., 0.]]);
    }
}
