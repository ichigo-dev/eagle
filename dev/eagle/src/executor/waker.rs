//------------------------------------------------------------------------------
/// # Waker
//------------------------------------------------------------------------------

use std::mem::ManuallyDrop;
use std::sync::Arc;
use std::task::{ RawWaker, RawWakerVTable, Waker };


//------------------------------------------------------------------------------
/// # waker_fn
///
/// Creates a new Waker from a closure.
//------------------------------------------------------------------------------
pub(super) fn waker_fn<F: Fn() + Send + Sync + 'static>( f: F ) -> Waker
{
    let raw = Arc::into_raw(Arc::new(f)) as *const ();
    let vtable = &WakerHelper::<F>::VTABLE;
    unsafe { Waker::from_raw(RawWaker::new(raw, vtable)) }
}


//------------------------------------------------------------------------------
/// # WakerHelper
//------------------------------------------------------------------------------
struct WakerHelper<F>(F);

impl<F: Fn() + Send + Sync + 'static> WakerHelper<F>
{
    const VTABLE: RawWakerVTable = RawWakerVTable::new
    (
        Self::clone_waker,
        Self::wake,
        Self::wake_by_ref,
        Self::drop_waker,
    );

    //--------------------------------------------------------------------------
    /// Clones the Waker.
    //--------------------------------------------------------------------------
    unsafe fn clone_waker( ptr: *const () ) -> RawWaker
    {
        let arc = ManuallyDrop::new(Arc::from_raw(ptr as *const F));
        let cloned_arc = arc.clone();
        std::mem::forget(arc);
        let ptr = Arc::into_raw(cloned_arc.into()) as *const ();
        RawWaker::new(ptr, &Self::VTABLE)
    }

    //--------------------------------------------------------------------------
    /// Wakes the Waker.
    //--------------------------------------------------------------------------
    unsafe fn wake( ptr: *const () )
    {
        let arc = Arc::from_raw(ptr as *const F);
        (arc)();
    }

    //--------------------------------------------------------------------------
    /// Wakes the Waker by reference.
    //--------------------------------------------------------------------------
    unsafe fn wake_by_ref( ptr: *const () )
    {
        let arc = ManuallyDrop::new(Arc::from_raw(ptr as *const F));
        (arc)();
        std::mem::forget(arc);
    }

    //--------------------------------------------------------------------------
    /// Drops the Waker.
    //--------------------------------------------------------------------------
    unsafe fn drop_waker( ptr: *const () )
    {
        drop(Arc::from_raw(ptr as *const F));
    }
}
