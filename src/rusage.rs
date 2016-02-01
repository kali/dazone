use std::io;
use std::io::Write;
use libc::{getrusage, RUSAGE_SELF, rusage, timeval};
use std::time::Duration;
use time;
use std::sync;

quick_error! {
    #[derive(Debug)]
    pub enum ResourceUsageError {
        Io(err: ::std::io::Error) { from() }
    }
}

pub type Result<T> = ::std::result::Result<T, ResourceUsageError>;

#[derive(Debug)]
pub struct ResourceUsage {
    pub virtual_size: u64,
    pub resident_size: u64,
    pub resident_size_max: u64,
    pub user_time: f64,
    pub system_time: f64,
    pub minor_fault: u64,
    pub major_fault: u64,
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
pub fn get_memory_usage() -> Result<ResourceUsage> {
    let info = darwin::task_info();
    let rusage = get_rusage();
    Ok(ResourceUsage {
        virtual_size: info.virtual_size,
        resident_size: info.resident_size,
        resident_size_max: info.resident_size_max,
        user_time: rusage.ru_utime.tv_sec as f64 + rusage.ru_utime.tv_usec as f64 / 1_000_000f64,
        system_time: rusage.ru_stime.tv_sec as f64 + rusage.ru_stime.tv_usec as f64 / 1_000_000f64,
        minor_fault: rusage.ru_minflt as u64,
        major_fault: rusage.ru_majflt as u64,
    })
}

#[cfg(target_os="linux")]
pub fn get_memory_usage() -> Result<ResourceUsage> {
    use std::fs::File;
    use std::io::Read;
    let mut proc_stat = String::new();
    let _ = try!(try!(File::open("/proc/self/stat")).read_to_string(&mut proc_stat));
    let mut tokens = proc_stat.split(" ");
    let rusage = get_rusage();
    Ok(ResourceUsage {
        virtual_size: tokens.nth(22).unwrap().parse().unwrap_or(0),
        resident_size: 4 * 1024 * tokens.next().unwrap().parse().unwrap_or(0),
        resident_size_max: 1024 * rusage.ru_maxrss as u64,
        user_time: rusage.ru_utime.tv_sec as f64 + rusage.ru_utime.tv_usec as f64 / 1_000_000f64,
        system_time: rusage.ru_stime.tv_sec as f64 + rusage.ru_stime.tv_usec as f64 / 1_000_000f64,
        minor_fault: rusage.ru_minflt as u64,
        major_fault: rusage.ru_majflt as u64,
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

#[derive(Debug,Default)]
pub struct Monitor {
    pub progress: sync::atomic::AtomicUsize,
}

impl Monitor {
    pub fn new<W:Write+Send+'static>(interval:time::Duration, write:W) -> sync::Arc<Monitor> {
        use std::sync::atomic::Ordering::Relaxed;
        let monitor = sync::Arc::new(Monitor::default());
        let monitor_to_go = monitor.clone();
        let cpus = ::num_cpus::get();
        ::std::thread::spawn(move || {
            let mut buffed = io::BufWriter::new(write);
            let started = time::get_time();
            for step in 0.. {
                let now = interval * step;
                let usage = get_memory_usage().unwrap();
                write!(buffed,
                       "{:7.3} {:4} {:10} {:2} {:8.3} {:8.3} {:10} {:10}\n",
                       now.num_milliseconds() as f32 / 1000f32,
                       monitor_to_go.progress.load(Relaxed),
                       usage.resident_size,
                       cpus,
                       usage.user_time,
                       usage.system_time,
                       usage.minor_fault, usage.major_fault,
                       )
                    .unwrap();
                buffed.flush().unwrap();
                let delay = started + now + interval - time::get_time();
                ::std::thread::sleep(Duration::from_millis(delay.num_milliseconds() as u64));
            }
        });
        monitor
    }

    pub fn set_progress(&self, progress:usize) {
        self.progress.store(progress, sync::atomic::Ordering::Relaxed);
    }

    pub fn add_progress(&self, progress:usize) {
        self.progress.fetch_add(progress, sync::atomic::Ordering::Relaxed);
    }
}

pub fn dump_memory_stats() {
    let _ = get_memory_usage().map(|usage| {
        println!("task_info: vsz:{} rsz:{} rszmax:{}",
                 usage.virtual_size,
                 usage.resident_size,
                 usage.resident_size_max)
    });
}
