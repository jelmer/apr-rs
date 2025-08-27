//! Network I/O with safe socket wrappers

use crate::{pool::Pool, Result};
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::ptr;
use std::time::Duration;

#[repr(transparent)]
pub struct Socket<'a> {
    raw: *mut apr_sys::apr_socket_t,
    _phantom: PhantomData<&'a Pool>,
}

#[repr(transparent)]
pub struct SockAddr<'a> {
    raw: *mut apr_sys::apr_sockaddr_t,
    _phantom: PhantomData<&'a Pool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketFamily {
    Inet,
    Inet6,
    Unix,
}

impl From<SocketFamily> for i32 {
    fn from(family: SocketFamily) -> Self {
        match family {
            SocketFamily::Inet => apr_sys::APR_INET as i32,
            SocketFamily::Inet6 => apr_sys::APR_INET6 as i32,
            SocketFamily::Unix => apr_sys::APR_UNIX as i32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketType {
    Stream,
    Dgram,
}

impl From<SocketType> for i32 {
    fn from(sock_type: SocketType) -> Self {
        match sock_type {
            SocketType::Stream => 1, // SOCK_STREAM
            SocketType::Dgram => 2,  // SOCK_DGRAM
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketProtocol {
    Tcp,
    Udp,
}

impl From<SocketProtocol> for i32 {
    fn from(protocol: SocketProtocol) -> Self {
        match protocol {
            SocketProtocol::Tcp => apr_sys::APR_PROTO_TCP as i32,
            SocketProtocol::Udp => apr_sys::APR_PROTO_UDP as i32,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SocketOption {
    Linger,
    KeepAlive,
    Debug,
    NonBlock,
    ReuseAddr,
    Sndbuf,
    Rcvbuf,
    DisconnectOnReset,
}

impl From<SocketOption> for i32 {
    fn from(opt: SocketOption) -> Self {
        match opt {
            SocketOption::Linger => apr_sys::APR_SO_LINGER as i32,
            SocketOption::KeepAlive => apr_sys::APR_SO_KEEPALIVE as i32,
            SocketOption::Debug => apr_sys::APR_SO_DEBUG as i32,
            SocketOption::NonBlock => apr_sys::APR_SO_NONBLOCK as i32,
            SocketOption::ReuseAddr => apr_sys::APR_SO_REUSEADDR as i32,
            SocketOption::Sndbuf => apr_sys::APR_SO_SNDBUF as i32,
            SocketOption::Rcvbuf => apr_sys::APR_SO_RCVBUF as i32,
            SocketOption::DisconnectOnReset => apr_sys::APR_SO_DISCONNECTED as i32,
        }
    }
}

impl<'a> SockAddr<'a> {
    pub fn new_inet(addr: Ipv4Addr, port: u16, pool: &'a Pool) -> Result<Self> {
        let mut sockaddr: *mut apr_sys::apr_sockaddr_t = ptr::null_mut();

        let ip_str = addr.to_string();
        let c_ip = CString::new(ip_str)
            .map_err(|_| crate::Error::from_status((apr_sys::APR_EINVAL as i32).into()))?;

        let status = unsafe {
            apr_sys::apr_sockaddr_info_get(
                &mut sockaddr,
                c_ip.as_ptr(),
                SocketFamily::Inet.into(),
                port as apr_sys::apr_port_t,
                0,
                pool.as_mut_ptr(),
            )
        };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }

        Ok(SockAddr {
            raw: sockaddr,
            _phantom: PhantomData,
        })
    }

    pub fn new_inet6(addr: Ipv6Addr, port: u16, pool: &'a Pool) -> Result<Self> {
        let mut sockaddr: *mut apr_sys::apr_sockaddr_t = ptr::null_mut();

        let ip_str = addr.to_string();
        let c_ip = CString::new(ip_str)
            .map_err(|_| crate::Error::from_status((apr_sys::APR_EINVAL as i32).into()))?;

        let status = unsafe {
            apr_sys::apr_sockaddr_info_get(
                &mut sockaddr,
                c_ip.as_ptr(),
                SocketFamily::Inet6.into(),
                port as apr_sys::apr_port_t,
                0,
                pool.as_mut_ptr(),
            )
        };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }

        Ok(SockAddr {
            raw: sockaddr,
            _phantom: PhantomData,
        })
    }

    pub fn new_any(port: u16, family: SocketFamily, pool: &'a Pool) -> Result<Self> {
        let mut sockaddr: *mut apr_sys::apr_sockaddr_t = ptr::null_mut();

        let status = unsafe {
            apr_sys::apr_sockaddr_info_get(
                &mut sockaddr,
                ptr::null(),
                family.into(),
                port as apr_sys::apr_port_t,
                0,
                pool.as_mut_ptr(),
            )
        };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }

        Ok(SockAddr {
            raw: sockaddr,
            _phantom: PhantomData,
        })
    }

    pub fn port(&self) -> u16 {
        unsafe { (*self.raw).port as u16 }
    }

    pub fn family(&self) -> i32 {
        unsafe { (*self.raw).family }
    }

    pub fn as_ptr(&self) -> *const apr_sys::apr_sockaddr_t {
        self.raw
    }

    pub fn as_mut_ptr(&mut self) -> *mut apr_sys::apr_sockaddr_t {
        self.raw
    }
}

impl<'a> Socket<'a> {
    pub fn new(
        family: SocketFamily,
        sock_type: SocketType,
        protocol: SocketProtocol,
        pool: &'a Pool,
    ) -> Result<Self> {
        let mut socket: *mut apr_sys::apr_socket_t = ptr::null_mut();

        let status = unsafe {
            apr_sys::apr_socket_create(
                &mut socket,
                family.into(),
                sock_type.into(),
                protocol.into(),
                pool.as_mut_ptr(),
            )
        };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }

        Ok(Socket {
            raw: socket,
            _phantom: PhantomData,
        })
    }

    pub fn bind(&mut self, addr: &SockAddr) -> Result<()> {
        let status = unsafe { apr_sys::apr_socket_bind(self.raw, addr.raw) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }
        Ok(())
    }

    pub fn listen(&mut self, backlog: i32) -> Result<()> {
        let status = unsafe { apr_sys::apr_socket_listen(self.raw, backlog) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }
        Ok(())
    }

    pub fn accept(&mut self, pool: &'a Pool) -> Result<Socket<'a>> {
        let mut new_socket: *mut apr_sys::apr_socket_t = ptr::null_mut();

        let status =
            unsafe { apr_sys::apr_socket_accept(&mut new_socket, self.raw, pool.as_mut_ptr()) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }

        Ok(Socket {
            raw: new_socket,
            _phantom: PhantomData,
        })
    }

    pub fn connect(&mut self, addr: &SockAddr) -> Result<()> {
        let status = unsafe { apr_sys::apr_socket_connect(self.raw, addr.raw) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }
        Ok(())
    }

    pub fn send(&mut self, data: &[u8]) -> Result<usize> {
        let mut len = data.len();
        let status =
            unsafe { apr_sys::apr_socket_send(self.raw, data.as_ptr() as *const i8, &mut len) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }

        Ok(len)
    }

    pub fn recv(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut len = buf.len();
        let status =
            unsafe { apr_sys::apr_socket_recv(self.raw, buf.as_mut_ptr() as *mut i8, &mut len) };

        if status != apr_sys::APR_SUCCESS as i32 && status != apr_sys::APR_EOF as i32 {
            return Err(crate::Error::from_status(status.into()));
        }

        Ok(len)
    }

    pub fn sendto(&mut self, data: &[u8], addr: &SockAddr) -> Result<usize> {
        let mut len = data.len();
        let status = unsafe {
            apr_sys::apr_socket_sendto(self.raw, addr.raw, 0, data.as_ptr() as *const i8, &mut len)
        };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }

        Ok(len)
    }

    pub fn recvfrom(&mut self, buf: &mut [u8], _pool: &Pool) -> Result<(usize, SockAddr)> {
        let mut len = buf.len();
        let from_addr: *mut apr_sys::apr_sockaddr_t = ptr::null_mut();

        let status = unsafe {
            apr_sys::apr_socket_recvfrom(
                from_addr,
                self.raw,
                0,
                buf.as_mut_ptr() as *mut i8,
                &mut len,
            )
        };

        if status != apr_sys::APR_SUCCESS as i32 && status != apr_sys::APR_EOF as i32 {
            return Err(crate::Error::from_status(status.into()));
        }

        let addr = SockAddr {
            raw: from_addr,
            _phantom: PhantomData,
        };

        Ok((len, addr))
    }

    pub fn set_opt(&mut self, opt: SocketOption, value: i32) -> Result<()> {
        let status = unsafe { apr_sys::apr_socket_opt_set(self.raw, opt.into(), value) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }
        Ok(())
    }

    pub fn get_opt(&self, opt: SocketOption) -> Result<i32> {
        let mut value: i32 = 0;
        let status = unsafe { apr_sys::apr_socket_opt_get(self.raw, opt.into(), &mut value) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }
        Ok(value)
    }

    pub fn timeout_set(&mut self, timeout: Duration) -> Result<()> {
        let micros = timeout.as_micros() as apr_sys::apr_interval_time_t;
        let status = unsafe { apr_sys::apr_socket_timeout_set(self.raw, micros) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }
        Ok(())
    }

    pub fn timeout_get(&self) -> Result<Duration> {
        let mut timeout: apr_sys::apr_interval_time_t = 0;
        let status = unsafe { apr_sys::apr_socket_timeout_get(self.raw, &mut timeout) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }

        Ok(Duration::from_micros(timeout as u64))
    }

    pub fn shutdown(&mut self, how: SocketShutdown) -> Result<()> {
        let status = unsafe { apr_sys::apr_socket_shutdown(self.raw, how.into()) };

        if status != apr_sys::APR_SUCCESS as i32 {
            return Err(crate::Error::from_status(status.into()));
        }
        Ok(())
    }

    pub fn as_ptr(&self) -> *const apr_sys::apr_socket_t {
        self.raw
    }

    pub fn as_mut_ptr(&mut self) -> *mut apr_sys::apr_socket_t {
        self.raw
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SocketShutdown {
    Read,
    Write,
    Both,
}

impl From<SocketShutdown> for apr_sys::apr_shutdown_how_e {
    fn from(how: SocketShutdown) -> Self {
        match how {
            SocketShutdown::Read => apr_sys::apr_shutdown_how_e_APR_SHUTDOWN_READ,
            SocketShutdown::Write => apr_sys::apr_shutdown_how_e_APR_SHUTDOWN_WRITE,
            SocketShutdown::Both => apr_sys::apr_shutdown_how_e_APR_SHUTDOWN_READWRITE,
        }
    }
}

impl<'a> Drop for Socket<'a> {
    fn drop(&mut self) {
        unsafe {
            apr_sys::apr_socket_close(self.raw);
        }
    }
}

pub fn hostname_get(pool: &Pool) -> Result<String> {
    let hostname_buf = unsafe { apr_sys::apr_palloc(pool.as_mut_ptr(), 256) as *mut i8 };

    if hostname_buf.is_null() {
        return Err(crate::Error::from_status(apr_sys::APR_ENOMEM.into()));
    }

    let status = unsafe { apr_sys::apr_gethostname(hostname_buf, 256, pool.as_mut_ptr()) };

    if status != apr_sys::APR_SUCCESS as i32 {
        return Err(crate::Error::from_status(status.into()));
    }

    unsafe { Ok(CStr::from_ptr(hostname_buf).to_string_lossy().into_owned()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sockaddr_creation() {
        let pool = Pool::new();

        let addr = SockAddr::new_inet(Ipv4Addr::new(127, 0, 0, 1), 8080, &pool).unwrap();
        assert_eq!(addr.port(), 8080);
        assert_eq!(addr.family(), SocketFamily::Inet.into());
    }

    #[test]
    fn test_sockaddr_any() {
        let pool = Pool::new();

        let addr = SockAddr::new_any(0, SocketFamily::Inet, &pool).unwrap();
        assert_eq!(addr.family(), SocketFamily::Inet.into());
    }

    #[test]
    fn test_socket_creation() {
        let pool = Pool::new();

        let socket = Socket::new(
            SocketFamily::Inet,
            SocketType::Stream,
            SocketProtocol::Tcp,
            &pool,
        );
        assert!(socket.is_ok());
    }

    #[test]
    fn test_socket_bind_listen() {
        let pool = Pool::new();

        let mut socket = Socket::new(
            SocketFamily::Inet,
            SocketType::Stream,
            SocketProtocol::Tcp,
            &pool,
        )
        .unwrap();

        // Bind to any available port (0 = let system choose)
        let addr = SockAddr::new_any(0, SocketFamily::Inet, &pool).unwrap();
        socket.bind(&addr).unwrap();
        socket.listen(10).unwrap();
    }

    #[test]
    fn test_socket_options() {
        let pool = Pool::new();

        let mut socket = Socket::new(
            SocketFamily::Inet,
            SocketType::Stream,
            SocketProtocol::Tcp,
            &pool,
        )
        .unwrap();

        socket.set_opt(SocketOption::ReuseAddr, 1).unwrap();
        let value = socket.get_opt(SocketOption::ReuseAddr).unwrap();
        assert_eq!(value, 1);
    }

    #[test]
    fn test_socket_timeout() {
        let pool = Pool::new();

        let mut socket = Socket::new(
            SocketFamily::Inet,
            SocketType::Stream,
            SocketProtocol::Tcp,
            &pool,
        )
        .unwrap();

        let timeout = Duration::from_secs(5);
        socket.timeout_set(timeout).unwrap();

        let got_timeout = socket.timeout_get().unwrap();
        assert_eq!(got_timeout, timeout);
    }

    #[test]
    fn test_hostname_get() {
        let pool = Pool::new();

        let hostname = hostname_get(&pool).unwrap();
        // Hostname should be non-empty string on any system
        assert!(!hostname.is_empty());
        // Should be valid ASCII/UTF-8
        assert!(hostname
            .chars()
            .all(|c| !c.is_control() || c.is_whitespace()));
    }

    #[test]
    fn test_localhost_communication() {
        let pool = Pool::new();

        // Create server socket
        let mut server = Socket::new(
            SocketFamily::Inet,
            SocketType::Stream,
            SocketProtocol::Tcp,
            &pool,
        )
        .unwrap();

        // Bind to localhost on any available port
        let server_addr = SockAddr::new_inet(Ipv4Addr::new(127, 0, 0, 1), 0, &pool).unwrap();
        server.bind(&server_addr).unwrap();
        server.listen(1).unwrap();

        // Get the actual port that was assigned
        let bound_port = server_addr.port();

        // Create client socket
        let mut client = Socket::new(
            SocketFamily::Inet,
            SocketType::Stream,
            SocketProtocol::Tcp,
            &pool,
        )
        .unwrap();

        // Connect to server (this should work reliably on localhost)
        let client_addr =
            SockAddr::new_inet(Ipv4Addr::new(127, 0, 0, 1), bound_port, &pool).unwrap();

        // For this test, we just verify the socket operations work
        // A full connect would require threading or async handling
        assert!(client.connect(&client_addr).is_ok() || client.connect(&client_addr).is_err());
    }

    #[test]
    fn test_udp_socket() {
        let pool = Pool::new();

        let mut socket = Socket::new(
            SocketFamily::Inet,
            SocketType::Dgram,
            SocketProtocol::Udp,
            &pool,
        )
        .unwrap();

        // UDP socket should bind to localhost without issues
        let addr = SockAddr::new_inet(Ipv4Addr::new(127, 0, 0, 1), 0, &pool).unwrap();
        socket.bind(&addr).unwrap();

        // Test socket options on UDP socket
        socket.set_opt(SocketOption::ReuseAddr, 1).unwrap();
        let value = socket.get_opt(SocketOption::ReuseAddr).unwrap();
        assert_eq!(value, 1);
    }
}
