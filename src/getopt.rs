//! Command line option parsing.
use crate::pool::Pool;
use std::marker::PhantomData;

/// A trait for types that can be converted into a sequence of allowed option characters.
pub trait IntoAllowedOptionChars {
    /// Converts the value into an iterator of characters that are allowed as options.
    fn into_iter(self) -> impl Iterator<Item = char>;
}

impl IntoAllowedOptionChars for &str {
    fn into_iter(self) -> impl std::iter::Iterator<Item = char> {
        self.chars().collect::<Vec<_>>().into_iter()
    }
}

impl IntoAllowedOptionChars for &[char] {
    fn into_iter(self) -> impl std::iter::Iterator<Item = char> {
        self.iter().copied()
    }
}

/// A command line option.
pub struct Option<'pool> {
    ptr: *mut apr_sys::apr_getopt_option_t,
    _pool: PhantomData<&'pool Pool<'pool>>,
}

/// An indicator for a command line option.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Indicator {
    /// A sentinel value.
    Sentinel,

    /// A single letter.
    Letter(char),

    /// An identifier.
    Identifier(i32),
}

impl From<Indicator> for i32 {
    fn from(indicator: Indicator) -> Self {
        match indicator {
            Indicator::Sentinel => 0,
            Indicator::Letter(c) => c as i32,
            Indicator::Identifier(i) => 255 + i,
        }
    }
}

impl From<i32> for Indicator {
    fn from(indicator: i32) -> Self {
        if indicator == 0 {
            Indicator::Sentinel
        } else if indicator > 255 {
            Indicator::Identifier(indicator - 255)
        } else {
            Indicator::Letter(indicator as u8 as char)
        }
    }
}

impl<'pool> Option<'pool> {
    /// Create a new option.
    pub fn new(
        pool: &'pool crate::pool::Pool<'pool>,
        name: &str,
        has_arg: bool,
        indicator: Indicator,
        description: std::option::Option<&'pool str>,
    ) -> Self {
        let name = std::ffi::CString::new(name).unwrap();
        let description = description.map(|s| std::ffi::CString::new(s).unwrap());

        let option = pool.calloc::<apr_sys::apr_getopt_option_t>();
        unsafe {
            (*option).name = name.as_ptr() as *mut _;
            (*option).has_arg = if has_arg { 1 } else { 0 };
            (*option).optch = indicator.into();
            if let Some(description) = description {
                (*option).description = description.as_ptr() as *mut _;
            }
        }

        Self {
            ptr: option,
            _pool: PhantomData,
        }
    }

    /// Returns the name of the option.
    pub fn name(&self) -> &str {
        unsafe {
            let name = (*self.ptr).name;
            std::ffi::CStr::from_ptr(name).to_str().unwrap()
        }
    }

    /// Returns true if the option has an argument.
    pub fn has_arg(&self) -> bool {
        unsafe { (*self.ptr).has_arg != 0 }
    }

    /// Returns the character code of the option.
    pub fn optch(&self) -> std::option::Option<u8> {
        unsafe {
            let v = (*self.ptr).optch;
            if v > 255 {
                None
            } else {
                Some(v as u8)
            }
        }
    }

    /// Returns the description of the option.
    pub fn description(&self) -> &str {
        unsafe {
            let description = (*self.ptr).description;
            std::ffi::CStr::from_ptr(description).to_str().unwrap()
        }
    }

    /// Returns the pointer to the underlying `apr_getopt_option_t` structure.
    pub fn as_ptr(&self) -> *const apr_sys::apr_getopt_option_t {
        self.ptr
    }

    /// Returns the mutable pointer to the underlying `apr_getopt_option_t` structure.
    pub fn as_mut_ptr(&mut self) -> *mut apr_sys::apr_getopt_option_t {
        self.ptr
    }
}

/// A command line option parser.
pub struct Getopt<'pool> {
    ptr: *mut apr_sys::apr_getopt_t,
    _pool: Pool<'pool>, // Owns the pool to keep it alive
}

/// The result of parsing a command line option.
pub enum GetoptResult {
    /// An option.
    Option(Indicator, std::option::Option<String>),

    /// A missing argument.
    MissingArgument(char),

    /// A bad option.
    BadOption(char),

    /// The end of options.
    End,
}

impl Getopt<'_> {
    /// Create a new `Getopt` instance.
    pub fn new(args: &[&str]) -> Result<Self, crate::Status> {
        let mut os = std::ptr::null_mut();
        let pool = crate::pool::Pool::new();

        let argv = args
            .iter()
            .map(|s| {
                let s = std::ffi::CString::new(*s).unwrap();
                unsafe { apr_sys::apr_pstrdup(pool.as_mut_ptr(), s.as_ptr()) }
            })
            .collect::<Vec<_>>();

        let rv = unsafe {
            apr_sys::apr_getopt_init(
                &mut os,
                pool.as_mut_ptr(),
                args.len() as i32,
                argv.as_slice().as_ptr() as *mut _,
            )
        };

        let status = crate::Status::from(rv);
        if status.is_success() {
            Ok(Self {
                ptr: os,
                _pool: pool,
            })
        } else {
            Err(status)
        }
    }

    /// Return the arguments.
    pub fn args(&self) -> Vec<&str> {
        unsafe {
            let args = (*self.ptr).argv;
            let args = std::slice::from_raw_parts(args, (*self.ptr).argc as usize);
            args.iter()
                .map(|&s| std::ffi::CStr::from_ptr(s).to_str().unwrap())
                .collect::<Vec<_>>()
        }
    }

    /// Allow interleaving of options and arguments.
    pub fn allow_interleaving(&mut self, allow: bool) {
        unsafe {
            (*self.ptr).interleave = if allow { 1 } else { 0 };
        }
    }

    /// Skip the first `skip` arguments.
    pub fn skip_start(&mut self, skip: i32) {
        unsafe {
            (*self.ptr).skip_start = skip;
        }
    }

    /// Skip the last `skip` arguments.
    pub fn skip_end(&mut self, skip: i32) {
        unsafe {
            (*self.ptr).skip_end = skip;
        }
    }

    /// Parse a command line option.
    pub fn getopt(&mut self, opts: impl IntoAllowedOptionChars) -> GetoptResult {
        let mut opts: Vec<std::ffi::c_char> =
            opts.into_iter().map(|c| c as std::ffi::c_char).collect();
        opts.push(0);
        let mut option_ch = 0;
        let mut option_arg: *const std::ffi::c_char = std::ptr::null_mut();

        let rv = unsafe {
            apr_sys::apr_getopt(
                self.ptr,
                opts.as_slice().as_ptr(),
                &mut option_ch,
                &mut option_arg,
            )
        };

        match rv as u32 {
            apr_sys::APR_SUCCESS => {
                let option_ch = option_ch as u8;
                let option_arg = if option_arg.is_null() {
                    None
                } else {
                    Some(
                        unsafe { std::ffi::CStr::from_ptr(option_arg) }
                            .to_str()
                            .unwrap()
                            .to_owned(),
                    )
                };
                GetoptResult::Option(Indicator::Letter(option_ch as char), option_arg)
            }
            apr_sys::APR_EOF => GetoptResult::End,
            apr_sys::APR_BADCH => GetoptResult::BadOption(option_ch as u8 as char),
            apr_sys::APR_BADARG => GetoptResult::MissingArgument(option_ch as u8 as char),
            _ => panic!("unexpected status: {}", rv),
        }
    }

    /// Parse a long command line option.
    pub fn getopt_long(&mut self, opts: &[Option]) -> GetoptResult {
        let mut option_ch: i32 = 0;
        let mut option_arg: *const std::ffi::c_char = std::ptr::null();
        let mut opts = opts
            .iter()
            .map(|o| o.as_ptr())
            .map(|ptr| unsafe { *ptr })
            .collect::<Vec<_>>();
        // sentinel
        opts.push(apr_sys::apr_getopt_option_t {
            name: std::ptr::null(),
            has_arg: 0,
            optch: 0,
            description: std::ptr::null(),
        });

        let rv = unsafe {
            apr_sys::apr_getopt_long(
                self.ptr,
                opts.as_slice().as_ptr(),
                &mut option_ch,
                &mut option_arg,
            )
        };

        match rv as u32 {
            apr_sys::APR_SUCCESS => {
                let option_arg = if option_arg.is_null() {
                    None
                } else {
                    Some(
                        unsafe { std::ffi::CStr::from_ptr(option_arg) }
                            .to_str()
                            .unwrap()
                            .to_owned(),
                    )
                };
                GetoptResult::Option(option_ch.into(), option_arg)
            }
            apr_sys::APR_EOF => GetoptResult::End,
            apr_sys::APR_BADCH => GetoptResult::BadOption(option_ch as u8 as char),
            apr_sys::APR_BADARG => GetoptResult::MissingArgument(option_ch as u8 as char),
            _ => panic!("unexpected status: {}", rv),
        }
    }

    /// Return ptr to the underlying `apr_getopt_t` structure.
    pub fn as_ptr(&self) -> *const apr_sys::apr_getopt_t {
        self.ptr
    }

    /// Return mutable ptr to the underlying `apr_getopt_t` structure.
    pub fn as_mut_ptr(&mut self) -> *mut apr_sys::apr_getopt_t {
        self.ptr
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_getopt_long() {
        let pool = crate::pool::Pool::new();
        let args = vec!["test", "-a", "-b", "foo", "-c", "bar"];
        let mut getopt = crate::getopt::Getopt::new(&args).unwrap();
        assert_eq!(getopt.args(), &args[..]);
        let opts = vec![
            crate::getopt::Option::new(&pool, "a", false, super::Indicator::Letter('a'), None),
            crate::getopt::Option::new(&pool, "b", true, super::Indicator::Letter('b'), None),
            crate::getopt::Option::new(&pool, "c", true, super::Indicator::Letter('c'), None),
        ];
        let mut got = vec![];
        loop {
            match getopt.getopt_long(&opts) {
                super::GetoptResult::Option(ch, arg) => got.push((ch, arg)),
                super::GetoptResult::End => break,
                super::GetoptResult::BadOption(o) => panic!("unexpected option: {}", o),
                super::GetoptResult::MissingArgument(o) => panic!("missing argument: {}", o),
            }
        }
        assert_eq!(
            got,
            vec![
                (super::Indicator::Letter('a'), None),
                (super::Indicator::Letter('b'), Some("foo".to_owned())),
                (super::Indicator::Letter('c'), Some("bar".to_owned()))
            ]
        );
    }

    #[test]
    fn test_getopt() {
        let args = vec!["test", "-a", "-b", "foo", "-c", "bar"];
        let mut getopt = crate::getopt::Getopt::new(&args).unwrap();
        assert_eq!(getopt.args(), &args[..]);
        getopt.allow_interleaving(true);
        getopt.skip_start(1);
        getopt.skip_end(1);
        let mut got = vec![];
        loop {
            match getopt.getopt("ab:c:") {
                super::GetoptResult::Option(ch, arg) => got.push((ch, arg)),
                super::GetoptResult::End => break,
                _ => panic!("unexpected result"),
            }
        }

        assert_eq!(
            got,
            vec![
                (super::Indicator::Letter('a'), None),
                (super::Indicator::Letter('b'), Some("foo".to_owned())),
                (super::Indicator::Letter('c'), Some("bar".to_owned()))
            ]
        );
    }
}
