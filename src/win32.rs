use std::ffi::{c_long, c_ulong, c_void};
use std::ptr::null_mut;
use winapi::shared::ntdef::{HANDLE, LUID};
use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
use winapi::um::securitybaseapi::AdjustTokenPrivileges;
use winapi::um::winbase::LookupPrivilegeValueA;
use winapi::um::winnt::{LUID_AND_ATTRIBUTES, SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY};

//https://gist.github.com/bitshifter/c87aa396446bbebeab29
//https://github.com/Kudaes/rust_tips_and_tricks?tab=readme-ov-file#DInvoke_rs
#[repr(u32)]
enum SYSTEM_INFORMATION_CLASS {
    SystemMemoryListInformation = 80
}
type PVOID = *mut c_void;
type ULONG = c_ulong;
type LONG = c_long;
type NTSTATUS = LONG;

#[repr(u32)]
enum SYSTEM_MEMORY_LIST_COMMAND
{
    MemoryCaptureAccessedBits,
    MemoryCaptureAndResetAccessedBits,
    MemoryEmptyWorkingSets,
    MemoryFlushModifiedList,
    MemoryPurgeStandbyList,
    MemoryPurgeLowPriorityStandbyList,
    MemoryCommandMax
}

pub type NtSetSystemInformation = unsafe extern "system" fn (SYSTEM_INFORMATION_CLASS, PVOID, ULONG) -> NTSTATUS;


pub struct Win32 {

}

impl Win32 {
    fn adjust_process_privilege() -> i32 {
        //https://github.com/trickster0/OffensiveRust/blob/master/EnableDebugPrivileges/src/main.rs
        unsafe{
            let mut h_token: HANDLE = 0 as _;
            OpenProcessToken(GetCurrentProcess(),TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,&mut h_token);
            let privs = LUID_AND_ATTRIBUTES {Luid: LUID { LowPart: 0, HighPart: 0,},Attributes: SE_PRIVILEGE_ENABLED,};
            let mut tp = TOKEN_PRIVILEGES {PrivilegeCount: 1,Privileges: [privs ;1],};
            let privilege = "SeProfileSingleProcessPrivilege\0";
            let _ = LookupPrivilegeValueA(null_mut(),privilege.as_ptr() as *const i8,&mut tp.Privileges[0].Luid,);
            let r = AdjustTokenPrivileges(h_token,0,&mut tp,size_of::<TOKEN_PRIVILEGES>() as _,null_mut(),null_mut());

            return r;
        }
    }

    #[cfg(target_os = "windows")]
    pub fn clear_standby_list() -> bool {
        Win32::adjust_process_privilege();

        let command = SYSTEM_MEMORY_LIST_COMMAND::MemoryPurgeStandbyList;
        let c: PVOID = unsafe { std::mem::transmute(&command) };

        let result = Win32::nt_set_system_information(SYSTEM_INFORMATION_CLASS::SystemMemoryListInformation,
                                               c, size_of::<SYSTEM_MEMORY_LIST_COMMAND>() as ULONG
        );
        return result == 0;
    }

    fn nt_set_system_information(system_information_class: SYSTEM_INFORMATION_CLASS,
                                     system_information: PVOID,
                                     system_information_length: ULONG) -> NTSTATUS {
        unsafe
        {
            let ret: Option<i32>;
            let func_ptr: NtSetSystemInformation;
            let ntdll = dinvoke_rs::dinvoke::get_module_base_address("ntdll.dll");
            dinvoke_rs::dinvoke::dynamic_invoke!(ntdll,"NtSetSystemInformation",func_ptr,ret,system_information_class, system_information, system_information_length);

            return match ret {
                Some(x) => x,
                None => -1,
            }
        }
    }
}