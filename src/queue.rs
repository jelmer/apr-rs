//! Thread-safe FIFO queue implementation.
//!
//! This module provides a thread-safe, bounded FIFO queue that can be used
//! for inter-thread communication. The queue blocks on push when full and
//! blocks on pop when empty.

use crate::{pool::Pool, Error, Result, Status};
use std::ffi::c_void;
use std::marker::PhantomData;
use std::ptr;

/// A thread-safe FIFO queue that stores raw pointers.
///
/// This is a direct wrapper around APR's queue implementation.
/// The queue is bounded and will block on push operations when full,
/// and block on pop operations when empty.
///
/// Values are stored as raw pointers - the queue does not manage their lifetime.
pub struct Queue<'pool> {
    ptr: *mut apr_sys::apr_queue_t,
    _phantom: PhantomData<&'pool Pool>,
}

impl<'pool> Queue<'pool> {
    /// Create a new queue with the specified capacity.
    ///
    /// # Arguments
    /// * `capacity` - Maximum number of elements the queue can hold
    /// * `pool` - Memory pool for allocation
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

    /// Create a queue from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be valid and point to an APR queue.
    pub unsafe fn from_ptr(ptr: *mut apr_sys::apr_queue_t) -> Self {
        Self {
            ptr,
            _phantom: PhantomData,
        }
    }

    /// Push a raw pointer onto the queue.
    ///
    /// This will block if the queue is full until space becomes available
    /// or the queue is interrupted.
    ///
    /// # Safety
    /// The caller must ensure the pointer remains valid until it is popped from the queue.
    pub unsafe fn push(&mut self, data: *mut c_void) -> Result<()> {
        let status = apr_sys::apr_queue_push(self.ptr, data);

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(Error::from_status(Status::from(status)));
        }

        Ok(())
    }

    /// Try to push a raw pointer onto the queue without blocking.
    ///
    /// Returns an error if the queue is full.
    ///
    /// # Safety
    /// The caller must ensure the pointer remains valid until it is popped from the queue.
    pub unsafe fn try_push(&mut self, data: *mut c_void) -> Result<()> {
        let status = apr_sys::apr_queue_trypush(self.ptr, data);

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(Error::from_status(Status::from(status)));
        }

        Ok(())
    }

    /// Pop a raw pointer from the queue.
    ///
    /// This will block if the queue is empty until an element becomes available
    /// or the queue is interrupted.
    pub fn pop(&mut self) -> Result<*mut c_void> {
        let mut data: *mut c_void = ptr::null_mut();

        let status = unsafe { apr_sys::apr_queue_pop(self.ptr, &mut data) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(Error::from_status(Status::from(status)));
        }

        Ok(data)
    }

    /// Try to pop a raw pointer from the queue without blocking.
    ///
    /// Returns an error if the queue is empty.
    pub fn try_pop(&mut self) -> Result<*mut c_void> {
        let mut data: *mut c_void = ptr::null_mut();

        let status = unsafe { apr_sys::apr_queue_trypop(self.ptr, &mut data) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(Error::from_status(Status::from(status)));
        }

        Ok(data)
    }

    /// Get the current number of elements in the queue.
    pub fn size(&self) -> u32 {
        unsafe { apr_sys::apr_queue_size(self.ptr) }
    }

    /// Check if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.size() == 0
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

// Since Queue holds raw pointers, we need to be explicit about thread safety
unsafe impl<'pool> Send for Queue<'pool> {}
unsafe impl<'pool> Sync for Queue<'pool> {}

impl<'pool> std::fmt::Debug for Queue<'pool> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Queue")
            .field("size", &self.size())
            .field("ptr", &self.ptr)
            .finish()
    }
}

/// A type-safe queue for passing references between threads.
///
/// This wrapper ensures that references passed through the queue
/// have the appropriate lifetime constraints.
pub struct TypedQueue<'pool, T> {
    inner: Queue<'pool>,
    _phantom: PhantomData<T>,
}

impl<'pool, T> TypedQueue<'pool, T> {
    /// Create a new typed queue.
    pub fn new(capacity: u32, pool: &'pool Pool) -> Result<Self> {
        Ok(TypedQueue {
            inner: Queue::new(capacity, pool)?,
            _phantom: PhantomData,
        })
    }

    /// Create a typed queue from an existing raw APR queue pointer.
    ///
    /// # Safety
    /// The caller must ensure:
    /// - The pointer is valid and points to an APR queue
    /// - The queue contains pointers to values of type T
    /// - The queue outlives 'pool
    pub unsafe fn from_ptr(ptr: *mut apr_sys::apr_queue_t) -> Self {
        Self {
            inner: Queue::from_ptr(ptr),
            _phantom: PhantomData,
        }
    }

    /// Push a reference onto the queue.
    ///
    /// The reference must outlive the pool.
    pub fn push_ref(&mut self, data: &'pool T) -> Result<()> {
        unsafe { self.inner.push(data as *const T as *mut c_void) }
    }

    /// Try to push a reference without blocking.
    pub fn try_push_ref(&mut self, data: &'pool T) -> Result<()> {
        unsafe { self.inner.try_push(data as *const T as *mut c_void) }
    }

    /// Pop a reference from the queue.
    pub fn pop_ref(&mut self) -> Result<&'pool T> {
        let ptr = self.inner.pop()?;
        Ok(unsafe { &*(ptr as *const T) })
    }

    /// Try to pop a reference without blocking.
    pub fn try_pop_ref(&mut self) -> Result<&'pool T> {
        let ptr = self.inner.try_pop()?;
        Ok(unsafe { &*(ptr as *const T) })
    }

    /// Get the current number of elements.
    pub fn size(&self) -> u32 {
        self.inner.size()
    }

    /// Check if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Interrupt all waiting threads.
    pub fn interrupt_all(&mut self) -> Result<()> {
        self.inner.interrupt_all()
    }

    /// Terminate the queue.
    pub fn terminate(&mut self) -> Result<()> {
        self.inner.terminate()
    }
}

unsafe impl<'pool, T: Send> Send for TypedQueue<'pool, T> {}
unsafe impl<'pool, T: Send> Sync for TypedQueue<'pool, T> {}

/// A queue for passing owned values between threads.
///
/// This wrapper manages the lifetime of heap-allocated values
/// passed through the queue, ensuring they are properly cleaned up.
pub struct BoxedQueue<'pool, T: Send> {
    inner: Queue<'pool>,
    _phantom: PhantomData<T>,
}

impl<'pool, T: Send> BoxedQueue<'pool, T> {
    /// Create a new boxed queue.
    pub fn new(capacity: u32, pool: &'pool Pool) -> Result<Self> {
        Ok(BoxedQueue {
            inner: Queue::new(capacity, pool)?,
            _phantom: PhantomData,
        })
    }

    /// Push a value onto the queue.
    ///
    /// The value is boxed and ownership is transferred to the queue.
    pub fn push(&mut self, value: T) -> Result<()> {
        let boxed = Box::new(value);
        let ptr = Box::into_raw(boxed);
        unsafe { self.inner.push(ptr as *mut c_void) }
    }

    /// Try to push a value without blocking.
    pub fn try_push(&mut self, value: T) -> Result<()> {
        let boxed = Box::new(value);
        let ptr = Box::into_raw(boxed);
        match unsafe { self.inner.try_push(ptr as *mut c_void) } {
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
        let ptr = self.inner.pop()?;
        Ok(*unsafe { Box::from_raw(ptr as *mut T) })
    }

    /// Try to pop a value without blocking.
    pub fn try_pop(&mut self) -> Result<T> {
        let ptr = self.inner.try_pop()?;
        Ok(*unsafe { Box::from_raw(ptr as *mut T) })
    }

    /// Get the current number of elements.
    pub fn size(&self) -> u32 {
        self.inner.size()
    }

    /// Check if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Interrupt all waiting threads.
    pub fn interrupt_all(&mut self) -> Result<()> {
        self.inner.interrupt_all()
    }

    /// Terminate the queue.
    pub fn terminate(&mut self) -> Result<()> {
        self.inner.terminate()
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
        let mut queue = Queue::new(10, &pool).unwrap();

        assert!(queue.is_empty());
        assert_eq!(queue.size(), 0);

        // Push some values
        let val1 = 42i32;
        let val2 = 84i32;

        unsafe {
            queue.push(&val1 as *const i32 as *mut c_void).unwrap();
            queue.push(&val2 as *const i32 as *mut c_void).unwrap();
        }

        assert_eq!(queue.size(), 2);
        assert!(!queue.is_empty());

        // Pop values
        let out1 = queue.pop().unwrap();
        let out2 = queue.pop().unwrap();

        unsafe {
            assert_eq!(*(out1 as *const i32), 42);
            assert_eq!(*(out2 as *const i32), 84);
        }

        assert!(queue.is_empty());
    }

    #[test]
    fn test_typed_queue() {
        let pool = Pool::new();
        let mut queue = TypedQueue::<i32>::new(10, &pool).unwrap();

        let val1 = 42;
        let val2 = 84;

        queue.push_ref(&val1).unwrap();
        queue.push_ref(&val2).unwrap();

        assert_eq!(queue.size(), 2);

        let out1 = queue.pop_ref().unwrap();
        let out2 = queue.pop_ref().unwrap();

        assert_eq!(*out1, 42);
        assert_eq!(*out2, 84);
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
    fn test_queue_try_operations() {
        let pool = Pool::new();
        let mut queue = BoxedQueue::<i32>::new(1, &pool).unwrap();

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
