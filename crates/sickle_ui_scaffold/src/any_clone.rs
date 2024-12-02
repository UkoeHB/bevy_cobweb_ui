use std::any::TypeId;
use std::fmt::Debug;

use dyn_clone::DynClone;

//-------------------------------------------------------------------------------------------------------------------

/// Trait mirroring `Any` that allows dyn-compatible clone, and includes `Debug + Send + Sync + 'static` bounds.
pub trait AnyClone: DynClone + Debug + Send + Sync + 'static
{
    fn type_id(&self) -> TypeId;
}
impl<T: DynClone + Debug + Send + Sync + 'static> AnyClone for T
{
    fn type_id(&self) -> TypeId
    {
        TypeId::of::<T>()
    }
}

impl dyn AnyClone
{
    pub fn is<T: AnyClone>(&self) -> bool
    {
        let t = TypeId::of::<T>();
        let concrete = self.type_id();
        t == concrete
    }

    pub fn downcast_ref<T: AnyClone>(&self) -> Option<&T>
    {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented AnyClone for all types; no other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(self.downcast_ref_unchecked()) }
        } else {
            None
        }
    }
    pub fn downcast_mut<T: AnyClone>(&mut self) -> Option<&mut T>
    {
        if self.is::<T>() {
            // SAFETY: just checked whether we are pointing to the correct type, and we can rely on
            // that check for memory safety because we have implemented AnyClone for all types; no other
            // impls can exist as they would conflict with our impl.
            unsafe { Some(self.downcast_mut_unchecked()) }
        } else {
            None
        }
    }

    /// # Safety
    ///
    /// The contained value must be of type `T`. Calling this method
    /// with the incorrect type is *undefined behavior*.
    unsafe fn downcast_ref_unchecked<T: AnyClone>(&self) -> &T
    {
        debug_assert!(self.is::<T>());
        // SAFETY: caller guarantees that T is the correct type
        unsafe { &*(self as *const dyn AnyClone as *const T) }
    }
    /// # Safety
    ///
    /// The contained value must be of type `T`. Calling this method
    /// with the incorrect type is *undefined behavior*.
    unsafe fn downcast_mut_unchecked<T: AnyClone>(&mut self) -> &mut T
    {
        debug_assert!(self.is::<T>());
        // SAFETY: caller guarantees that T is the correct type
        unsafe { &mut *(self as *mut dyn AnyClone as *mut T) }
    }
}

//-------------------------------------------------------------------------------------------------------------------
