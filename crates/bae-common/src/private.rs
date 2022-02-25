use std::iter::FromIterator;

use syn::Result;

pub use super::from_attributes_meta;

/// Collect all `syn::Result` in this iterator, combining all `syn::Error`
pub trait IterCombineSynErrors<T, I>
where
    I: std::iter::Iterator<Item = Result<T>>,
{
    /// Collect all `syn::Result` in this iterator, combining all `syn::Error`
    fn collect_syn_error<B: FromIterator<T>>(self) -> Result<B>
    where
        B: Default;
}

impl<T, I> IterCombineSynErrors<T, I> for I
where
    I: std::iter::Iterator<Item = Result<T>>,
{
    fn collect_syn_error<B: FromIterator<T>>(self) -> Result<B>
    where
        B: Default,
    {
        let res_vec =
            self.fold::<Result<Vec<T>>, _>(Ok(Default::default()), |accum, res| {
                match (accum, res) {
                    (Err(mut ea), Err(er)) => {
                        ea.combine(er);
                        Err(ea)
                    }
                    (Err(ea), Ok(_)) => Err(ea),
                    (Ok(_), Err(er)) => Err(er),
                    (Ok(mut va), Ok(vr)) => {
                        va.push(vr);
                        Ok(va)
                    }
                }
            });

        res_vec.map(|vec| B::from_iter(vec))
    }
}
