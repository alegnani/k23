use crate::{AddressRangeExt, Error, Mode, VirtualAddress};
use core::marker::PhantomData;
use core::ops::Range;

#[must_use]
pub struct Flush<M> {
    asid: usize,
    range: Option<Range<VirtualAddress>>,
    _m: PhantomData<M>,
}

impl<M: Mode> Flush<M> {
    pub fn empty(asid: usize) -> Self {
        Self {
            asid,
            range: None,
            _m: PhantomData,
        }
    }

    pub fn new(asid: usize, range: Range<VirtualAddress>) -> Self {
        Self {
            asid,
            range: Some(range),
            _m: PhantomData,
        }
    }

    /// Flush the range of virtual addresses from the TLB.
    ///
    /// # Errors
    ///
    /// Returns an error if the range could not be flushed due to an underlying hardware error.
    pub fn flush(self) -> crate::Result<()> {
        log::trace!("flushing range {:?}", self.range);
        if let Some(range) = self.range {
            M::invalidate_range(self.asid, range)?;
        } else {
            log::warn!("attempted to flush empty range, ignoring");
        }

        Ok(())
    }

    /// # Safety
    ///
    /// Not flushing after mutating the page translation tables will likely lead to unintended
    /// consequences such as inconsistent views of the address space between different harts.
    ///
    /// You should only call this if you know what you're doing.
    pub unsafe fn ignore(self) {}

    /// Extend the range to include the given range.
    ///
    /// # Errors
    ///
    /// Returns an error if the given ASID does not match the ASID of this `Flush`.
    pub fn extend_range(&mut self, asid: usize, other: Range<VirtualAddress>) -> crate::Result<()> {
        if self.asid == asid {
            if let Some(this) = self.range.take() {
                self.range = Some(this.concat(other));
            } else {
                self.range = Some(other);
            }

            Ok(())
        } else {
            Err(Error::AddressSpaceMismatch {
                expected: self.asid,
                found: asid,
            })
        }
    }
}