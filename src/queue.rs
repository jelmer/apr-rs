//! Thread-safe FIFO queue implementation.
//!
//! This module provides a thread-safe, bounded FIFO queue that can be used
//! for inter-thread communication. The queue blocks on push when full and
//! blocks on pop when empty.

use crate::{pool::Pool, Error, Result, Status};
use std::marker::PhantomData;
use std::ptr;

/// Trait for types that can be retrieved from APR queues.
pub trait FromAprQueueElement<'queue>: Sized {
    /// Convert from queue element pointer to this type.
    ///
    /// # Safety
    /// The caller must ensure:
    /// - The pointer is valid or NULL
    /// - If non-NULL, the data lives at least as long as 'queue lifetime
    unsafe fn from_apr_queue_element(ptr: *mut std::ffi::c_void) -> Self;
}

/// Trait for types that can be stored in APR queues.
/// The lifetime 'a represents the minimum lifetime of the data being queued.
pub trait IntoAprQueueElement<'a> {
    /// Convert this value into a pointer for storage in an APR queue.
    fn into_apr_queue_element(&self) -> *mut std::ffi::c_void;
}

// Implementation for references - panics on NULL
impl<'queue, T> FromAprQueueElement<'queue> for &'queue T {
    unsafe fn from_apr_queue_element(ptr: *mut std::ffi::c_void) -> Self {
        if ptr.is_null() {
            panic!("Cannot convert NULL pointer to reference. Use Option<&T> if NULL values are expected.");
        }
        &*(ptr as *const T)
    }
}

// Implementation for Option<&T> - gracefully handles NULL
impl<'queue, T> FromAprQueueElement<'queue> for Option<&'queue T> {
    unsafe fn from_apr_queue_element(ptr: *mut std::ffi::c_void) -> Self {
        if ptr.is_null() {
            None
        } else {
            Some(&*(ptr as *const T))
        }
    }
}

// Implementation for raw pointers - passes through NULL
impl<'queue, T> FromAprQueueElement<'queue> for *mut T {
    unsafe fn from_apr_queue_element(ptr: *mut std::ffi::c_void) -> Self {
        ptr as *mut T
    }
}

impl<'queue, T> FromAprQueueElement<'queue> for *const T {
    unsafe fn from_apr_queue_element(ptr: *mut std::ffi::c_void) -> Self {
        ptr as *const T
    }
}

impl<'a, T> IntoAprQueueElement<'a> for *mut T {
    fn into_apr_queue_element(&self) -> *mut std::ffi::c_void {
        *self as *mut std::ffi::c_void
    }
}

impl<'a, T> IntoAprQueueElement<'a> for *const T {
    fn into_apr_queue_element(&self) -> *mut std::ffi::c_void {
        *self as *mut T as *mut std::ffi::c_void
    }
}

// Implementation for references - ensures data outlives the queue
impl<'a, T> IntoAprQueueElement<'a> for &'a T {
    fn into_apr_queue_element(&self) -> *mut std::ffi::c_void {
        *self as *const T as *mut T as *mut std::ffi::c_void
    }
}

impl<'a, T> IntoAprQueueElement<'a> for &'a mut T {
    fn into_apr_queue_element(&self) -> *mut std::ffi::c_void {
        *self as *const T as *mut T as *mut std::ffi::c_void
    }
}

// Implementation for Option<&T> - converts None to NULL
impl<'a, T> IntoAprQueueElement<'a> for Option<&'a T> {
    fn into_apr_queue_element(&self) -> *mut std::ffi::c_void {
        match self {
            Some(ptr) => *ptr as *const T as *mut T as *mut std::ffi::c_void,
            None => std::ptr::null_mut(),
        }
    }
}

impl<'a, T> IntoAprQueueElement<'a> for Option<&'a mut T> {
    fn into_apr_queue_element(&self) -> *mut std::ffi::c_void {
        match self {
            Some(ptr) => *ptr as *const T as *mut T as *mut std::ffi::c_void,
            None => std::ptr::null_mut(),
        }
    }
}

/// A thread-safe FIFO queue.
///
/// The queue is bounded and will block on push operations when full,
/// and block on pop operations when empty. It's designed for passing
/// pointers between threads safely.
pub struct Queue<'pool, T> {
    ptr: *mut apr_sys::apr_queue_t,
    _phantom: PhantomData<(T, &'pool Pool)>,
}

impl<'pool, T> Queue<'pool, T> {
    /// Create a new queue from a raw pointer.
    pub fn from_ptr(ptr: *mut apr_sys::apr_queue_t) -> Self {
        Self {
            ptr,
            _phantom: PhantomData,
        }
    }

    /// Create a new queue with the specified capacity.
    ///
    /// # Arguments
    /// * `capacity` - Maximum number of elements the queue can hold
    /// * `pool` - Memory pool for allocation
    ///
    /// # Example
    /// ```
    /// use apr::Pool;
    /// use apr::queue::Queue;
    ///
    /// let pool = Pool::new();
    /// let queue: Queue<i32> = Queue::new(100, &pool).unwrap();
    /// ```
    pub fn new(capacity: u32, pool: &'pool Pool) -> Result<Self> {
        let mut queue_ptr: *mut apr_sys::apr_queue_t = ptr::null_mut();

        let status =
            unsafe { apr_sys::apr_queue_create(&mut queue_ptr, capacity, pool.as_mut_ptr()) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(Error::from_status(Status::from(status)));
        }

        Ok(Queue {
            ptr: queue_ptr,
            _phantom: PhantomData,
        })
    }

    /// Get the current number of elements in the queue.
    pub fn size(&self) -> u32 {
        unsafe { apr_sys::apr_queue_size(self.ptr) }
    }

    /// Interrupt all threads blocked on this queue.
    ///
    /// All threads blocked on push or pop operations will wake up and
    /// receive an error.
    pub fn interrupt_all(&mut self) -> Result<()> {
        let status = unsafe { apr_sys::apr_queue_interrupt_all(self.ptr) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(Error::from_status(Status::from(status)));
        }

        Ok(())
    }

    /// Terminate the queue.
    ///
    /// This wakes up all blocked threads and prevents further operations.
    pub fn terminate(&mut self) -> Result<()> {
        let status = unsafe { apr_sys::apr_queue_term(self.ptr) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(Error::from_status(Status::from(status)));
        }

        Ok(())
    }

    /// Check if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    /// Get a raw pointer to the underlying apr_queue_t.
    ///
    /// # Safety
    /// The pointer is only valid for the lifetime of this Queue instance.
    pub unsafe fn as_ptr(&self) -> *const apr_sys::apr_queue_t {
        self.ptr
    }

    /// Get a mutable raw pointer to the underlying apr_queue_t.
    ///
    /// # Safety
    /// The pointer is only valid for the lifetime of this Queue instance.
    pub unsafe fn as_mut_ptr(&mut self) -> *mut apr_sys::apr_queue_t {
        self.ptr
    }
}

// Methods that require IntoAprQueueElement for pushing values
impl<'pool, 'data, T> Queue<'pool, T>
where
    T: IntoAprQueueElement<'data>,
    'data: 'pool, // Data must outlive the queue
{
    /// Push an element onto the queue.
    ///
    /// This will block if the queue is full until space becomes available
    /// or the queue is interrupted.
    pub fn push(&mut self, data: T) -> Result<()> {
        let status = unsafe { apr_sys::apr_queue_push(self.ptr, data.into_apr_queue_element()) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(Error::from_status(Status::from(status)));
        }

        Ok(())
    }

    /// Try to push an element onto the queue without blocking.
    ///
    /// Returns an error if the queue is full.
    pub fn try_push(&mut self, data: T) -> Result<()> {
        let status = unsafe { apr_sys::apr_queue_trypush(self.ptr, data.into_apr_queue_element()) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(Error::from_status(Status::from(status)));
        }

        Ok(())
    }
}

// Methods that require FromAprQueueElement for popping values
impl<'pool, T: FromAprQueueElement<'pool>> Queue<'pool, T> {
    /// Pop an element from the queue.
    ///
    /// This will block if the queue is empty until an element becomes available
    /// or the queue is interrupted.
    pub fn pop(&mut self) -> Result<T> {
        let mut data: *mut std::ffi::c_void = ptr::null_mut();

        let status = unsafe { apr_sys::apr_queue_pop(self.ptr, &mut data) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(Error::from_status(Status::from(status)));
        }

        Ok(unsafe { T::from_apr_queue_element(data) })
    }

    /// Try to pop an element from the queue without blocking.
    ///
    /// Returns an error if the queue is empty.
    pub fn try_pop(&mut self) -> Result<T> {
        let mut data: *mut std::ffi::c_void = ptr::null_mut();

        let status = unsafe { apr_sys::apr_queue_trypop(self.ptr, &mut data) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(Error::from_status(Status::from(status)));
        }

        Ok(unsafe { T::from_apr_queue_element(data) })
    }
}

// Since Queue holds raw pointers, we need to be explicit about thread safety
unsafe impl<'pool, T: Send> Send for Queue<'pool, T> {}
unsafe impl<'pool, T: Send> Sync for Queue<'pool, T> {}

impl<'pool, T> std::fmt::Debug for Queue<'pool, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Queue")
            .field("size", &self.size())
            .field("ptr", &self.ptr)
            .finish()
    }
}

/// A safer wrapper for passing boxed values through the queue.
///
/// This wrapper helps manage the lifetime of heap-allocated values
/// passed through the queue.
pub struct BoxedQueue<'pool, T: Send> {
    queue: Queue<'pool, *mut T>,
}

impl<'pool, T: Send> BoxedQueue<'pool, T> {
    /// Create a new boxed queue.
    pub fn new(capacity: u32, pool: &'pool Pool) -> Result<Self> {
        Ok(BoxedQueue {
            queue: Queue::new(capacity, pool)?,
        })
    }

    /// Push a value onto the queue.
    ///
    /// The value is boxed and ownership is transferred to the queue.
    pub fn push(&mut self, value: T) -> Result<()> {
        let boxed = Box::new(value);
        let ptr = Box::into_raw(boxed);
        self.queue.push(ptr)
    }

    /// Try to push a value without blocking.
    pub fn try_push(&mut self, value: T) -> Result<()> {
        let boxed = Box::new(value);
        let ptr = Box::into_raw(boxed);
        match self.queue.try_push(ptr) {
            Ok(()) => Ok(()),
            Err(e) => {
                // Reclaim the box if push failed
                unsafe {
                    drop(Box::from_raw(ptr));
                }
                Err(e)
            }
        }
    }

    /// Pop a value from the queue.
    ///
    /// Ownership of the value is transferred to the caller.
    pub fn pop(&mut self) -> Result<T> {
        let ptr: *mut T = self.queue.pop()?;
        Ok(*unsafe { Box::from_raw(ptr) })
    }

    /// Try to pop a value without blocking.
    pub fn try_pop(&mut self) -> Result<T> {
        let ptr: *mut T = self.queue.try_pop()?;
        Ok(*unsafe { Box::from_raw(ptr) })
    }

    /// Get the current number of elements in the queue.
    pub fn size(&self) -> u32 {
        self.queue.size()
    }

    /// Check if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Interrupt all waiting threads.
    pub fn interrupt_all(&mut self) -> Result<()> {
        self.queue.interrupt_all()
    }

    /// Terminate the queue.
    pub fn terminate(&mut self) -> Result<()> {
        self.queue.terminate()
    }
}

impl<'pool, T: Send> std::fmt::Debug for BoxedQueue<'pool, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoxedQueue")
            .field("size", &self.size())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pool;

    #[test]
    fn test_queue_basic() {
        let pool = Pool::new();
        let mut queue: Queue<*mut i32> = Queue::new(10, &pool).unwrap();

        assert!(queue.is_empty());
        assert_eq!(queue.size(), 0);

        // Push some values
        let val1 = Box::new(42);
        let val2 = Box::new(84);
        let ptr1 = Box::into_raw(val1);
        let ptr2 = Box::into_raw(val2);

        queue.push(ptr1).unwrap();
        queue.push(ptr2).unwrap();

        assert_eq!(queue.size(), 2);
        assert!(!queue.is_empty());

        // Pop values
        let out1 = queue.pop().unwrap();
        let out2 = queue.pop().unwrap();

        assert_eq!(unsafe { *Box::from_raw(out1) }, 42);
        assert_eq!(unsafe { *Box::from_raw(out2) }, 84);

        assert!(queue.is_empty());
    }

    #[test]
    fn test_queue_try_operations() {
        let pool = Pool::new();
        let mut queue: Queue<*mut i32> = Queue::new(2, &pool).unwrap();

        // Try pop on empty queue should fail
        assert!(queue.try_pop().is_err());

        // Fill the queue
        let val1 = Box::into_raw(Box::new(1));
        let val2 = Box::into_raw(Box::new(2));
        queue.try_push(val1).unwrap();
        queue.try_push(val2).unwrap();

        // Try push on full queue should fail
        let val3 = Box::into_raw(Box::new(3));
        assert!(queue.try_push(val3).is_err());
        unsafe {
            drop(Box::from_raw(val3));
        } // Clean up

        // Try pop should succeed
        let out = queue.try_pop().unwrap();
        assert_eq!(unsafe { *Box::from_raw(out) }, 1);

        // Clean up remaining value
        let out = queue.try_pop().unwrap();
        unsafe {
            drop(Box::from_raw(out));
        }
    }

    #[test]
    fn test_boxed_queue() {
        let pool = Pool::new();
        let mut queue = BoxedQueue::new(10, &pool).unwrap();

        assert!(queue.is_empty());

        // Push values
        queue.push(String::from("hello")).unwrap();
        queue.push(String::from("world")).unwrap();

        assert_eq!(queue.size(), 2);

        // Pop values
        let s1 = queue.pop().unwrap();
        let s2 = queue.pop().unwrap();

        assert_eq!(s1, "hello");
        assert_eq!(s2, "world");

        assert!(queue.is_empty());
    }

    #[test]
    fn test_boxed_queue_try_operations() {
        let pool = Pool::new();
        let mut queue = BoxedQueue::new(1, &pool).unwrap();

        // Try pop on empty should fail
        assert!(queue.try_pop().is_err());

        // Fill queue
        queue.try_push(42).unwrap();

        // Try push on full should fail
        assert!(queue.try_push(84).is_err());

        // Try pop should succeed
        let val = queue.try_pop().unwrap();
        assert_eq!(val, 42);
    }
}
