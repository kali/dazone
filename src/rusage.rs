use libc::{getrusage, RUSAGE_SELF, rusage, timeval};
use std::time::Duration;

quick_error! {
    #[derive(Debug)]
    pub enum MemoryUsageError {
        Io(err: ::std::io::Error) { from() }
    }
}

pub type Result<T> = ::std::result::Result<T, MemoryUsageError>;

#[derive(Debug)]
pub struct MemoryUsage {
    pub virtual_size: u64,
    pub resident_size: u64,
    pub resident_size_max: u64,
}

#[cfg(target_os="macos")]
mod darwin {
    use libc::*;
    #[repr(C)]
    pub struct BasicTaskInfo {
        pub virtual_size: u64,
        pub resident_size: u64,
        pub resident_size_max: u64,
        pub user_time: timeval,
        pub system_time: timeval,
        pub policy: c_int,
        pub suspend_count: c_uint,
    }

    impl BasicTaskInfo {
        pub fn empty() -> BasicTaskInfo {
            BasicTaskInfo {
                virtual_size: 0,
                resident_size: 0,
                resident_size_max: 0,
                user_time: timeval {
                    tv_sec: 0,
                    tv_usec: 0,
                },
                system_time: timeval {
                    tv_sec: 0,
                    tv_usec: 0,
                },
                policy: 0,
                suspend_count: 0,
            }
        }
    }
    mod ffi {
        use libc::*;
        extern "C" {
            pub fn mach_task_self() -> c_uint;
            pub fn task_info(task: c_uint,
                             flavor: c_int,
                             task_info: *mut super::BasicTaskInfo,
                             count: *mut c_uint)
                             -> c_uint;
        }
    }
    pub fn task_self() -> c_uint {
        unsafe { ffi::mach_task_self() }
    }
    pub fn task_info() -> BasicTaskInfo {
        let mut info = BasicTaskInfo::empty();
        let mut count: c_uint =
            (::std::mem::size_of::<BasicTaskInfo>() / ::std::mem::size_of::<c_uint>()) as c_uint;
        unsafe {
            ffi::task_info(task_self(), 20, &mut info, &mut count);
        }
        info
    }
}

#[cfg(target_os="macos")]
pub fn get_memory_usage() -> Result<MemoryUsage> {
    let info = darwin::task_info();
    Ok(MemoryUsage {
        virtual_size: info.virtual_size,
        resident_size: info.resident_size,
        resident_size_max: info.resident_size_max,
    })
}

#[cfg(target_os="linux")]
pub fn get_memory_usage() -> Result<MemoryUsage> {
    use std::fs::File;
    use std::io::Read;
    let mut proc_stat = String::new();
    let _ = try!(try!(File::open("/proc/self/stat")).read_to_string(&mut proc_stat));
    let mut tokens = proc_stat.split(" ");
    Ok(MemoryUsage {
        virtual_size: tokens.nth(22).unwrap().parse().unwrap_or(0),
        resident_size: 4 * 1024 * tokens.next().unwrap().parse().unwrap_or(0),
        resident_size_max: 1024 * get_rusage().ru_maxrss as u64,
    })
}

pub fn get_rusage() -> rusage {
    let mut usage = rusage {
        ru_idrss: 0,
        ru_nvcsw: 0,
        ru_ixrss: 0,
        ru_isrss: 0,
        ru_inblock: 0,
        ru_minflt: 0,
        ru_oublock: 0,
        ru_nivcsw: 0,
        ru_stime: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        ru_nswap: 0,
        ru_maxrss: 0,
        ru_majflt: 0,
        ru_msgrcv: 0,
        ru_msgsnd: 0,
        ru_utime: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        ru_nsignals: 0,
    };
    unsafe {
        getrusage(RUSAGE_SELF, &mut usage);
    }
    usage
}

pub fn start_monitor(interval: Duration) {
    ::std::thread::spawn(move || {
        loop {
            dump_memory_stats();
            ::std::thread::sleep(interval);
        }
    });
}

pub fn dump_memory_stats() {
    let _ = get_memory_usage().map(|usage| {
        println!("task_info: vsz:{} rsz:{} rszmax:{}",
                 usage.virtual_size,
                 usage.resident_size,
                 usage.resident_size_max)
    });
}
