use std::marker::PhantomData;

pub fn array_windows<T, const N: usize>(slice: &[T]) -> ArrayWindows<'_, T, N> {
    assert_ne!(N, 0);
    ArrayWindows::new(slice)
}

/// A windowed iterator over a slice in overlapping chunks (`N` elements at a
/// time), starting at the beginning of the slice
///
/// This struct is created by the [`array_windows`] method on [slices].
///
/// # Example
///
/// ```
/// #![feature(array_windows)]
///
/// let slice = [0, 1, 2, 3];
/// let iter = slice.array_windows::<2>();
/// ```
///
/// [`array_windows`]: slice::array_windows
/// [slices]: slice
#[derive(Debug, Clone, Copy)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct ArrayWindows<'a, T: 'a, const N: usize> {
    slice_head: *const T,
    num: usize,
    marker: PhantomData<&'a [T; N]>,
}

impl<'a, T: 'a, const N: usize> ArrayWindows<'a, T, N> {
    #[inline]
    pub(super) fn new(slice: &'a [T]) -> Self {
        let num_windows = slice.len().saturating_sub(N - 1);
        Self {
            slice_head: slice.as_ptr(),
            num: num_windows,
            marker: PhantomData,
        }
    }
}

impl<'a, T, const N: usize> Iterator for ArrayWindows<'a, T, N> {
    type Item = &'a [T; N];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.num == 0 {
            return None;
        }
        // SAFETY:
        // This is safe because it's indexing into a slice guaranteed to be length > N.
        let ret = unsafe { &*self.slice_head.cast::<[T; N]>() };
        // SAFETY: Guaranteed that there are at least 1 item remaining otherwise
        // earlier branch would've been hit
        self.slice_head = unsafe { self.slice_head.add(1) };

        self.num -= 1;
        Some(ret)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.num, Some(self.num))
    }

    #[inline]
    fn count(self) -> usize {
        self.num
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if self.num <= n {
            self.num = 0;
            return None;
        }
        // SAFETY:
        // This is safe because it's indexing into a slice guaranteed to be length > N.
        let ret = unsafe { &*self.slice_head.add(n).cast::<[T; N]>() };
        // SAFETY: Guaranteed that there are at least n items remaining
        self.slice_head = unsafe { self.slice_head.add(n + 1) };

        self.num -= n + 1;
        Some(ret)
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        self.nth(self.num.checked_sub(1)?)
    }
}

impl<'a, T, const N: usize> DoubleEndedIterator for ArrayWindows<'a, T, N> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a [T; N]> {
        if self.num == 0 {
            return None;
        }
        // SAFETY: Guaranteed that there are n items remaining, n-1 for 0-indexing.
        let ret = unsafe { &*self.slice_head.add(self.num - 1).cast::<[T; N]>() };
        self.num -= 1;
        Some(ret)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<&'a [T; N]> {
        if self.num <= n {
            self.num = 0;
            return None;
        }
        // SAFETY: Guaranteed that there are n items remaining, n-1 for 0-indexing.
        let ret = unsafe { &*self.slice_head.add(self.num - (n + 1)).cast::<[T; N]>() };
        self.num -= n + 1;
        Some(ret)
    }
}

//  impl<T, const N: usize> ExactSizeIterator for ArrayWindows<'_, T, N> {
//      fn is_empty(&self) -> bool {
//          self.num == 0
//      }
//  }
