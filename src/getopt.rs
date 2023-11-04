use crate::pool::PooledPtr;

pub trait IntoAllowedOptionChars {
    fn into_iter(self) -> std::vec::IntoIter<char>;
}

impl IntoAllowedOptionChars for &str {
    fn into_iter(self) -> std::vec::IntoIter<char> {
        self.chars().collect::<Vec<_>>().into_iter()
    }
}

impl IntoAllowedOptionChars for &[char] {
    fn into_iter(self) -> std::vec::IntoIter<char> {
        self.to_vec().into_iter()
    }
}

pub struct Option<'pool>(
    PooledPtr<crate::generated::apr_getopt_option_t>,
    std::marker::PhantomData<&'pool ()>,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Indicator {
    Sentinel,
    Letter(char),
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
    pub fn new(
        name: &str,
        has_arg: bool,
        indicator: Indicator,
        description: std::option::Option<&'pool str>,
    ) -> Self {
        let name = std::ffi::CString::new(name).unwrap();
        let description = description.map(|s| std::ffi::CString::new(s).unwrap());

        Self(
            PooledPtr::initialize(|pool| unsafe {
                let mut option = pool.calloc::<crate::generated::apr_getopt_option_t>();
                (*option).name = name.as_ptr() as *mut _;
                (*option).has_arg = if has_arg { 1 } else { 0 };
                (*option).optch = indicator.into();
                if let Some(description) = description {
                    (*option).description = description.as_ptr() as *mut _;
                }
                Ok::<_, crate::Status>(option)
            })
            .unwrap(),
            std::marker::PhantomData,
        )
    }

    /// Returns the name of the option.
    pub fn name(&self) -> &str {
        unsafe {
            let name = (*self.0.as_ptr()).name;
            std::ffi::CStr::from_ptr(name).to_str().unwrap()
        }
    }

    /// Returns true if the option has an argument.
    pub fn has_arg(&self) -> bool {
        self.0.has_arg != 0
    }

    /// Returns the character code of the option.
    pub fn optch(&self) -> std::option::Option<u8> {
        let v = self.0.optch;
        if v > 255 {
            None
        } else {
            Some(v as u8)
        }
    }

    pub fn description(&self) -> &str {
        unsafe {
            let description = self.0.description;
            std::ffi::CStr::from_ptr(description).to_str().unwrap()
        }
    }

    pub fn as_ptr(&self) -> *const crate::generated::apr_getopt_option_t {
        self.0.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut crate::generated::apr_getopt_option_t {
        self.0.as_mut_ptr()
    }
}

pub struct Getopt(PooledPtr<crate::generated::apr_getopt_t>);

pub enum GetoptResult {
    Option(Indicator, std::option::Option<String>),
    MissingArgument(char),
    BadOption(char),
    End,
}

impl Getopt {
    pub fn new(args: &[&str]) -> Result<Self, crate::Status> {
        PooledPtr::initialize(|pool| unsafe {
            let mut os = std::ptr::null_mut();

            let argv = args
                .iter()
                .map(|s| {
                    let s = std::ffi::CString::new(*s).unwrap();
                    crate::generated::apr_pstrdup(pool.as_mut_ptr(), s.as_ptr())
                })
                .collect::<Vec<_>>();

            let rv = crate::generated::apr_getopt_init(
                &mut os,
                pool.as_mut_ptr(),
                args.len() as i32,
                argv.as_slice().as_ptr() as *mut _,
            );

            let status = crate::Status::from(rv);
            if status.is_success() {
                Ok(os)
            } else {
                Err(status)
            }
        })
        .map(Self)
    }

    pub fn args(&self) -> Vec<&str> {
        unsafe {
            let args = self.0.argv;
            let args = std::slice::from_raw_parts(args, self.0.argc as usize);
            args.iter()
                .map(|&s| std::ffi::CStr::from_ptr(s).to_str().unwrap())
                .collect::<Vec<_>>()
        }
    }

    pub fn allow_interleaving(&mut self, allow: bool) {
        self.0.interleave = if allow { 1 } else { 0 };
    }

    pub fn skip_start(&mut self, skip: i32) {
        self.0.skip_start = skip;
    }

    pub fn skip_end(&mut self, skip: i32) {
        self.0.skip_end = skip;
    }

    pub fn getopt(&mut self, opts: impl IntoAllowedOptionChars) -> GetoptResult {
        let mut opts: Vec<i8> = opts.into_iter().map(|c| c as i8).collect();
        opts.push(0);
        let mut option_ch = 0;
        let mut option_arg: *const i8 = std::ptr::null_mut();

        let rv = unsafe {
            crate::generated::apr_getopt(
                self.0.as_mut_ptr(),
                opts.as_slice().as_ptr(),
                &mut option_ch,
                &mut option_arg,
            )
        };

        match rv as u32 {
            crate::generated::APR_SUCCESS => {
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
            crate::generated::APR_EOF => GetoptResult::End,
            crate::generated::APR_BADCH => GetoptResult::BadOption(option_ch as u8 as char),
            crate::generated::APR_BADARG => GetoptResult::MissingArgument(option_ch as u8 as char),
            _ => panic!("unexpected status: {}", rv),
        }
    }

    pub fn getopt_long(&mut self, opts: &[Option]) -> GetoptResult {
        let mut option_ch: i32 = 0;
        let mut option_arg: *const i8 = std::ptr::null();
        let mut opts = opts
            .iter()
            .map(|o| o.as_ptr())
            .map(|ptr| unsafe { *ptr })
            .collect::<Vec<_>>();
        // sentinel
        opts.push(crate::generated::apr_getopt_option_t {
            name: std::ptr::null(),
            has_arg: 0,
            optch: 0,
            description: std::ptr::null(),
        });

        let rv = unsafe {
            crate::generated::apr_getopt_long(
                self.0.as_mut_ptr(),
                opts.as_slice().as_ptr(),
                &mut option_ch,
                &mut option_arg,
            )
        };

        match rv as u32 {
            crate::generated::APR_SUCCESS => {
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
            crate::generated::APR_EOF => GetoptResult::End,
            crate::generated::APR_BADCH => GetoptResult::BadOption(option_ch as u8 as char),
            crate::generated::APR_BADARG => GetoptResult::MissingArgument(option_ch as u8 as char),
            _ => panic!("unexpected status: {}", rv),
        }
    }

    pub fn as_ptr(&self) -> *const crate::generated::apr_getopt_t {
        self.0.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut crate::generated::apr_getopt_t {
        self.0.as_mut_ptr()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_getopt_long() {
        let args = vec!["test", "-a", "-b", "foo", "-c", "bar"];
        let mut getopt = crate::getopt::Getopt::new(&args).unwrap();
        assert_eq!(getopt.args(), &args[..]);
        let opts = vec![
            crate::getopt::Option::new("a", false, super::Indicator::Letter('a'), None),
            crate::getopt::Option::new("b", true, super::Indicator::Letter('b'), None),
            crate::getopt::Option::new("c", true, super::Indicator::Letter('c'), None),
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
