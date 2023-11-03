use crate::pool::PooledPtr;

pub struct Option<'pool>(PooledPtr<'pool, crate::generated::apr_getopt_option_t>);

impl<'pool> Option<'pool> {
    pub fn new(
        name: &str,
        has_arg: bool,
        optch: std::option::Option<u8>,
        description: std::option::Option<&str>,
    ) -> Self {
        let name = std::ffi::CString::new(name).unwrap();
        let description = description.map(|s| std::ffi::CString::new(s).unwrap());
        let optch: i32 = optch.map_or(300_i32, |c| c as i32);

        Self(
            PooledPtr::initialize(|pool| unsafe {
                let mut option = pool.calloc::<crate::generated::apr_getopt_option_t>();
                (*option).name = name.as_ptr() as *mut _;
                (*option).has_arg = if has_arg { 1 } else { 0 };
                (*option).optch = optch;
                if let Some(description) = description {
                    (*option).description = description.as_ptr() as *mut _;
                }
                Ok::<_, crate::Status>(option)
            })
            .unwrap(),
        )
    }

    pub fn name(&self) -> &str {
        unsafe {
            let name = (*self.0.as_ptr()).name;
            std::ffi::CStr::from_ptr(name).to_str().unwrap()
        }
    }

    pub fn has_arg(&self) -> bool {
        unsafe { (*self.0.as_ptr()).has_arg != 0 }
    }

    pub fn optch(&self) -> std::option::Option<u8> {
        let v = unsafe { (*self.0.as_ptr()).optch };
        if v > 255 {
            None
        } else {
            Some(v as u8)
        }
    }

    pub fn description(&self) -> &str {
        unsafe {
            let description = (*self.0.as_ptr()).description;
            std::ffi::CStr::from_ptr(description).to_str().unwrap()
        }
    }

    pub fn as_ptr(&self) -> *const crate::generated::apr_getopt_option_t {
        self.0.as_ptr()
    }
}

pub struct Getopt<'pool>(PooledPtr<'pool, crate::generated::apr_getopt_t>);

impl<'pool> Getopt<'pool> {
    pub fn new(args: &[&str]) -> Result<Self, crate::Status> {
        PooledPtr::initialize(|pool| unsafe {
            let mut os = std::ptr::null_mut();

            let argv = args
                .iter()
                .map(|s| std::ffi::CString::new(*s).unwrap())
                .collect::<Vec<_>>();

            let rv = crate::generated::apr_getopt_init(
                &mut os,
                pool.as_mut_ptr(),
                args.len() as i32,
                argv.as_ptr() as *mut _,
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

    pub fn allow_interleaving(&mut self, allow: bool) {
        self.0.interleave = if allow { 1 } else { 0 };
    }

    pub fn skip_start(&mut self, skip: i32) {
        self.0.skip_start = skip;
    }

    pub fn skip_end(&mut self, skip: i32) {
        self.0.skip_end = skip;
    }

    pub fn getopt(
        &mut self,
        opts: Vec<char>,
    ) -> Result<(char, std::option::Option<String>), crate::Status> {
        let opts: Vec<i8> = opts.into_iter().map(|c| c as i8).collect();
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

        let status = crate::Status::from(rv);
        if status.is_success() {
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
            Ok((option_ch as char, option_arg))
        } else {
            Err(status)
        }
    }

    pub fn getopt_long(
        &mut self,
        opts: &[Option],
    ) -> Result<(char, std::option::Option<String>), crate::Status> {
        let mut option_ch: i32 = 0;
        let mut option_arg: *const i8 = std::ptr::null();
        let opts = opts
            .iter()
            .map(|o| o.as_ptr())
            .map(|ptr| unsafe { *ptr })
            .collect::<Vec<_>>();
        let rv = unsafe {
            crate::generated::apr_getopt_long(
                self.0.as_mut_ptr(),
                opts.as_slice().as_ptr(),
                &mut option_ch,
                &mut option_arg,
            )
        };

        let status = crate::Status::from(rv);
        if status.is_success() {
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
            Ok((option_ch as char, option_arg))
        } else {
            Err(status)
        }
    }
}

#[cfg(test)]
mod tests {
    #[ignore = "crashing"]
    #[test]
    fn test_getopt_long() {
        let args = vec!["test", "-a", "-b", "foo", "-c", "bar"];
        let mut getopt = crate::getopt::Getopt::new(&args).unwrap();
        let opts = vec![
            crate::getopt::Option::new("a", false, None, None),
            crate::getopt::Option::new("b", true, None, None),
            crate::getopt::Option::new("c", true, None, None),
        ];
        let mut got = vec![];
        while let Ok((ch, arg)) = getopt.getopt_long(&opts) {
            got.push((ch, arg));
        }
        assert_eq!(
            got,
            vec![
                ('a', None),
                ('b', Some("foo".to_owned())),
                ('c', Some("bar".to_owned()))
            ]
        );
    }

    #[ignore = "crashing"]
    #[test]
    fn test_getopt() {
        let args = vec!["test", "-a", "-b", "foo", "-c", "bar"];
        let mut getopt = crate::getopt::Getopt::new(&args).unwrap();
        getopt.allow_interleaving(true);
        getopt.skip_start(1);
        getopt.skip_end(1);
        let mut got = vec![];
        while let Ok((ch, arg)) = getopt.getopt(vec!['a', 'b', 'c']) {
            got.push((ch, arg));
        }
        assert_eq!(
            got,
            vec![
                ('a', None),
                ('b', Some("foo".to_owned())),
                ('c', Some("bar".to_owned()))
            ]
        );
    }
}
