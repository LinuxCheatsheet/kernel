//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! An intrusive linked list implementation using `RawLink`s.
//!
//! An _intrusive_ list is a list structure wherein the type of element stored
//! in the list holds references to other nodes. This means that we don't have
//! to store a separate node data type that holds the stored elements and
//! pointers to other nodes, reducing the amount of memory allocated. We can
//! use intrusive lists in code that runs without the kernel memory allocator,
//! like the allocator implementation itself, since each list element manages
//! its own memory.
use super::rawlink::RawLink;

use core::marker::PhantomData;
use core::ptr::Unique;
#[cfg(test)] mod test;

pub unsafe trait OwnedRef<T> {
    unsafe fn from_raw(ptr: *mut T) -> Self;
    unsafe fn take(self);
    fn get(&self) -> &T;
    fn get_mut(&mut self) -> &mut T;
}

/// This trait defines a node in an intrusive list.
///
/// A Node must be capable of providing mutable and immutable references to
/// the previous and next nodes in the list.
pub trait Node: Sized {
    fn next(&self) -> &RawLink<Self>;
    fn prev(&self) -> &RawLink<Self>;

    fn next_mut(&mut self) -> &mut RawLink<Self>;
    fn prev_mut(&mut self) -> &mut RawLink<Self>;
}

/// The `List` struct is our way of interacting with an intrusive list.
///
/// It stores a pointer to the head and tail of the list, the length of the
/// list, and a `PhantomData` marker for the list's `OwnedRef` type. It
/// provides the methods for pushing, popping, and indexing the list.
pub struct List<T, N>
where T: OwnedRef<N>
    , N: Node {
    head: RawLink<N>
  , tail: RawLink<N>
  , _ty_marker: PhantomData<T>
  , length: usize
 }

 // impl<T> Node for List<T>
 // where T: OwnedRef
 //     , T: Node {
 //
 //    fn next(&self) -> &RawLink<Self> { &self.head }
 //    fn prev(&self) -> &RawLink<Self> { &self.tail }
 //
 //    fn next_mut(&mut self) -> &mut RawLink<Self> { self.head }
 //    fn prev_mut(&mut self) -> &mut RawLink<Self> { self.tail }
 // }
impl<T, N> List<T, N>
where T: OwnedRef<N>
    , N: Node {

    /// Construct a new `List<T, N>` with zero elements
    pub const fn new() -> Self {
        List { head: RawLink::none()
             , tail: RawLink::none()
             , _ty_marker: PhantomData
             , length: 0 }
    }

    /// Returns the length of the list
    #[inline] pub fn len(&self) -> usize {
        self.length
    }

    /// Borrows the first element of the list as an `Option`
    ///
    /// # Returns:
    ///   - `Some(&N)` if the list has elements
    ///   - `None` if the list is empty.
    #[inline] pub fn front(&self) -> Option<&N> {
        unsafe { self.head.resolve() }
    }


    /// Borrows the last element of the list as an `Option`
    ///
    /// # Returns:
    ///   - `Some(&N)` if the list has elements
    ///   - `None` if the list is empty.
    #[inline] pub fn back(&self) -> Option<&N> {
        unsafe { self.tail.resolve() }
    }

    /// Mutably borrows the first element of the list as an `Option`
    ///
    /// # Returns:
    ///   - `Some(&mut N)` if the list has elements
    ///   - `None` if the list is empty.
    #[inline] pub fn front_mut(&mut self) -> Option<&mut N> {
        unsafe { self.head.resolve_mut() }
    }

    /// Mutably borrows the last element of the list as an `Option`
    ///
    /// # Returns:
    ///   - `Some(&mut N)` if the list has elements
    ///   - `None` if the list is empty.
    #[inline] pub fn back_mut(&mut self) -> Option<&mut N> {
        unsafe { self.tail.resolve_mut() }
    }

    /// Returns true if the list is empty.
    #[inline] pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    /// Push an element to the front of the list.
    // TODO: should this really be called "prepend"?
    pub fn push_front(&mut self, mut item: T) {
        unsafe {
            match self.head.resolve_mut() {
                None => {
                    // If this node's head is empty, set the pushed item's
                    // links to None, and make this node's tail point to the
                    // pushed item
                    *item.get_mut().next_mut() = RawLink::none();
                    *item.get_mut().prev_mut() = RawLink::none();
                    self.tail = RawLink::some(item.get_mut());
                }
              , Some(head) => {
                    // If this node is not empty, set the pushed item's tail
                    // to point at the head node, and make the head node's tail
                    // point to the pushed item
                    *item.get_mut().next_mut() = RawLink::some(head);
                    *item.get_mut().prev_mut() = RawLink::none();
                    *head.prev_mut() = RawLink::some(item.get_mut());
                }
            }
            // then, set this node's head pointer to point to the pushed item
            self.head = RawLink::some(item.get_mut());
            item.take();
            self.length += 1;
        }
    }

    /// Push an element to the back of the list.
    //  TODO: should this really be called "append"?
    //  (the Rust standard library uses `append` to refer to the "drain all the
    //  elements of another list and push them to this list" operation, but I
    //  think that that function is more properly called `concat`...)
    pub fn push_back(&mut self, mut item: T) {
        unsafe {
            match self.tail.resolve_mut() {
                None => {
                    // If this node's tail is empty, set the pushed item's
                    // links to  None, and make this node's head point to the
                    // pushed item
                    *item.get_mut().next_mut() = RawLink::none();
                    *item.get_mut().prev_mut() = RawLink::none();
                    self.head = RawLink::some(item.get_mut());
                }
              , Some(tail) => {
                    // If this node is not empty, set the pushed item's head
                    // to point at the tail node, and make the tail node's head
                    // point to the pushed item
                    *item.get_mut().next_mut() = RawLink::none();
                    *item.get_mut().prev_mut() = RawLink::some(tail);
                    *tail.next_mut() = RawLink::some(item.get_mut());
                }
            }
            // then, set this node's head pointer to point to the pushed item
            self.tail = RawLink::some(item.get_mut());
            item.take();
            self.length += 1;
        }
    }

    /// Removes and returns the element at the front of the list.
    ///
    /// # Returns:
    ///   - `Some(T)` containing the element at the front of the list if the
    ///     list is not empty
    ///   - `None` if the list is empty
    pub fn pop_front(&mut self) -> Option<T> {
        unsafe {
            self.head.take().resolve_mut()
                .map(|head| {
                    // mem::swap( &mut self.head
                    //          , head.next_mut().resolve_mut()
                    //                .map(|next| next.prev_mut())
                    //                .unwrap_or(&mut RawLink::none()) );
                    match head.next_mut().resolve_mut() {
                        None => self.tail = RawLink::none()
                      , Some(next) => {
                            *next.prev_mut() = RawLink::none();
                            self.head = RawLink::some(next);
                        }
                    }
                    self.length -= 1;
                    T::from_raw(head)
                })
        }
    }

    /// Removes and returns the element at the end of the list.
    ///
    /// # Returns:
    ///   - `Some(T)` containing the element at the end of the list if the
    ///     list is not empty
    ///   - `None` if the list is empty
    pub fn pop_back(&mut self) -> Option<T> {
        unsafe {
            self.tail.take().resolve_mut()
                .map(|tail| {
                    match tail.prev_mut().resolve_mut() {
                        None => self.head = RawLink::none()
                      , Some(prev) => {
                            *prev.next_mut() = RawLink::none();
                            self.tail = RawLink::some(prev);
                        }
                    }
                    self.length -= 1;
                    T::from_raw(tail)
                })
        }
    }

    /// Borrows the element at the front of the list
    ///
    /// # Returns:
    ///   - `Some(&T)` containing the element at the end of the list if the
    ///     list is not empty
    ///   - `None` if the list is empty
    pub fn peek_front(&self) -> Option<&N> {
        unsafe { self.tail.resolve() }
    }

    /// Returns a cursor for iterating over or modifying the list.
    pub fn cursor<'a>(&'a mut self) -> ListCursor<'a, T, N> {
        ListCursor { list: self
                   , current: RawLink::none() }
    }

}

// TODO: can we implement `Iterator` for cursors?
pub struct ListCursor<'a, T, N>
where T: OwnedRef<N>
    , T: 'a
    , N: Node
    , N: 'a {
        list: &'a mut List<T, N>
      , current: RawLink<N>
}

impl<'a, T, N> ListCursor<'a, T, N>
where T: OwnedRef<N>
    , T: 'a
    , N: Node
    , N: 'a {

    pub fn next(&mut self) -> Option<&mut N> {
        unsafe {
            match self.current.take().resolve_mut() {
                None => self.list.head.resolve_mut()
                            .and_then(|head| {
                                self.current = RawLink::some(head);
                                self.current.resolve_mut()
                            })
              , Some(thing) => {
                    self.current = match thing.next_mut().resolve_mut() {
                        None => RawLink::none()
                      , Some(other_thing) => RawLink::some(other_thing)
                    };
                    self.current.resolve_mut()
                }
            }
        }
    }

    pub fn peek_next(&self) -> Option<&N> {
        unsafe {
            self.current.resolve()
                .map_or( self.list.front()
                       , |curr| curr.next().resolve())
        }
    }

    pub fn remove(&mut self) -> Option<T> {
        unsafe {
            match self.current.resolve_mut() {
                None    => self.list.pop_front()
              , Some(c) =>
                    c.next_mut().take().resolve_mut()
                     .map(|p| {
                        match p.next_mut().resolve_mut() {
                            None => self.list.tail = RawLink::some(c)
                          , Some(n) => {
                                *n.prev_mut() = RawLink::some(c);
                                *c.next_mut() = RawLink::some(n);
                            }
                        }
                        T::from_raw(p)
                    })
            }
        }
    }

    pub fn find_and_remove<P>(&mut self, predicate: P) -> Option<T>
    where P: Fn(&N) -> bool {
        while self.peek_next().is_some() {
            if predicate(self.peek_next().unwrap()) == true {
                return self.remove()
            } else {
                self.next();
            }
        }
        None
    }


}

// impl<'a, T, N> Iterator for ListCursor<'a, T, N>
// where T: OwnedRef<N>
//     , T: 'a
//     , N: Node
//     , N: 'a {
//     type Item = &'a mut N;
//
//     fn next<'b: 'a>(&'b mut self) -> Option<&'a mut N> {
//         self.next()
//     }
// }

//
// unsafe impl<T> OwnedRef for Unique<T> where T: Node {
//
//     #[inline]
//     fn take(self) {}
//
//     unsafe fn from_raw(ptr: *mut T) -> Self {
//         Unique::new(ptr)
//     }
// }
//
// unsafe impl<'a, T> OwnedRef<T> for &'a mut T {
//     #
//     #[inline] unsafe fn from_raw(raw: *mut T) -> &'a mut T {
//         &mut *raw
//     }
//
//     #[inline] unsafe fn take(self) {
//         forget(self);
//     }
// }
//

unsafe impl<T> OwnedRef<T> for Unique<T>  {
    #[inline]
    fn get(&self) -> &T { unsafe { self.get() } }

    #[inline]
    fn get_mut(&mut self) -> &mut T { unsafe { self.get_mut() } }

    #[inline]
    unsafe fn take(self) {}

    unsafe fn from_raw(ptr: *mut T) -> Self {
        Unique::new(ptr)
    }
}

#[cfg(any(test, feature = "use-std"))]
unsafe impl<T> OwnedRef<T> for ::std::boxed::Box<T> {

    fn get(&self) -> &T { &**self }
    fn get_mut(&mut self) -> &mut T { &mut **self }

    #[inline] unsafe fn take(self) {
        ::std::boxed::Box::into_raw(self);
    }

    unsafe fn from_raw(ptr: *mut T) -> Self {
        ::std::boxed::Box::from_raw(ptr)
    }
}
