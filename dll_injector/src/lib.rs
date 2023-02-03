use std::{
    mem::{size_of, transmute},
    ptr,
};

use windows::{
    core::PCWSTR,
    s, w,
    Win32::{
        Foundation::{CloseHandle, ERROR_SUCCESS, HANDLE, PSID},
        Security::{
            Authorization::{
                ConvertStringSidToSidW, GetNamedSecurityInfoW, SetEntriesInAclW,
                SetNamedSecurityInfoW, EXPLICIT_ACCESS_W, SET_ACCESS, SE_FILE_OBJECT,
                TRUSTEE_IS_SID, TRUSTEE_IS_WELL_KNOWN_GROUP, TRUSTEE_W,
            },
            ACL, DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR,
            SUB_CONTAINERS_AND_OBJECTS_INHERIT,
        },
        System::{
            Diagnostics::{
                Debug::WriteProcessMemory,
                ToolHelp::{
                    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, Thread32First,
                    Thread32Next, PROCESSENTRY32W, TH32CS_SNAPPROCESS, TH32CS_SNAPTHREAD,
                    THREADENTRY32,
                },
            },
            LibraryLoader::{GetModuleHandleW, GetProcAddress},
            Memory::{LocalFree, VirtualAllocEx, MEM_COMMIT, PAGE_READWRITE},
            SystemServices::{GENERIC_EXECUTE, GENERIC_READ, GENERIC_WRITE},
            Threading::{
                CreateRemoteThread, OpenProcess, OpenThread, ResumeThread, SuspendThread,
                PROCESS_CREATE_THREAD, PROCESS_VM_OPERATION, PROCESS_VM_WRITE, THREAD_ALL_ACCESS,
            },
        },
    },
};

use error::InjectorError;

pub mod error;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Process {
    pid: u32,
}

impl Process {
    pub fn find_process(starts_with: &str) -> Result<Option<Self>, InjectorError> {
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }?;

        let mut process_entry = PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        if unsafe { Process32FirstW(snapshot, &mut process_entry) }.as_bool() {
            while unsafe { Process32NextW(snapshot, &mut process_entry) }.as_bool() {
                let os_string = String::from_utf16(&process_entry.szExeFile[..starts_with.len()])?;
                if os_string.to_lowercase() == *starts_with.to_lowercase() {
                    return Ok(Some(Process {
                        pid: process_entry.th32ProcessID,
                    }));
                }
            }
        }

        Ok(None)
    }

    pub fn wait_for_process(starts_with: &str) -> Result<Self, InjectorError> {
        loop {
            if let Some(process) = Process::find_process(starts_with)? {
                break Ok(process);
            }
        }
    }

    fn foreach_thread(&self, callback: fn(HANDLE)) -> Result<(), InjectorError> {
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, self.pid) }?;

        let mut thread_entry = THREADENTRY32 {
            dwSize: size_of::<THREADENTRY32>() as u32,
            ..Default::default()
        };

        if unsafe { Thread32First(snapshot, &mut thread_entry) }.as_bool() {
            while unsafe { Thread32Next(snapshot, &mut thread_entry) }.as_bool() {
                if thread_entry.th32ThreadID != self.pid {
                    continue;
                }

                let thread =
                    unsafe { OpenThread(THREAD_ALL_ACCESS, false, thread_entry.th32ThreadID) }?;

                callback(thread);
            }
        }

        Ok(())
    }

    pub fn freeze(&self) -> Result<(), InjectorError> {
        self.foreach_thread(|thread| unsafe {
            SuspendThread(thread);
        })
    }

    pub fn unfreeze(&self) -> Result<(), InjectorError> {
        self.foreach_thread(|thread| unsafe {
            ResumeThread(thread);
        })
    }

    fn acl_file(dll_path: &[u16], access_string: PCWSTR) -> Result<(), InjectorError> {
        let mut current_acl: *mut ACL = ptr::null::<ACL>() as *mut _;
        let mut security_descriptor = PSECURITY_DESCRIPTOR::default();
        let mut security_identifier = PSID::default();

        let result = unsafe {
            GetNamedSecurityInfoW(
                PCWSTR::from_raw(dll_path.as_ptr()),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION,
                None,
                None,
                Some(&mut current_acl),
                None,
                &mut security_descriptor,
            )
        };

        if result != ERROR_SUCCESS {
            // todo: return err
            return Ok(());
        }

        if unsafe { ConvertStringSidToSidW(access_string, &mut security_identifier as *mut _) }
            .as_bool()
        {
            let explicit_access = EXPLICIT_ACCESS_W {
                grfAccessPermissions: GENERIC_READ | GENERIC_WRITE | GENERIC_EXECUTE,
                grfAccessMode: SET_ACCESS,
                grfInheritance: SUB_CONTAINERS_AND_OBJECTS_INHERIT,
                Trustee: TRUSTEE_W {
                    TrusteeForm: TRUSTEE_IS_SID,
                    TrusteeType: TRUSTEE_IS_WELL_KNOWN_GROUP,
                    ptstrName: unsafe { transmute(security_identifier) },
                    ..Default::default()
                },
            };

            let mut new_acl: *mut ACL = ptr::null::<ACL>() as *mut _;

            if unsafe {
                SetEntriesInAclW(Some(&[explicit_access]), Some(current_acl), &mut new_acl)
            } == ERROR_SUCCESS
            {
                unsafe {
                    SetNamedSecurityInfoW(
                        PCWSTR::from_raw(dll_path.as_ptr()),
                        SE_FILE_OBJECT,
                        DACL_SECURITY_INFORMATION,
                        None,
                        None,
                        Some(new_acl),
                        None,
                    )
                };

                unsafe { LocalFree(new_acl as isize) };
            }
        }

        unsafe { LocalFree(transmute(security_descriptor)) };

        Ok(())
    }

    pub fn inject_dll(&self, dll_path: &str) -> Result<(), InjectorError> {
        self.freeze()?;

        let process_handle = unsafe {
            OpenProcess(
                PROCESS_CREATE_THREAD | PROCESS_VM_OPERATION | PROCESS_VM_WRITE,
                false,
                self.pid,
            )
        }?;

        let mut path = dll_path.encode_utf16().collect::<Vec<u16>>();
        path.push(0x00); // null terminating

        Process::acl_file(&path, w!("S-1-15-2-1"))?;

        let path_mem = unsafe {
            VirtualAllocEx(
                process_handle,
                None,
                path.len() * 2,
                MEM_COMMIT,
                PAGE_READWRITE,
            )
        };

        if path_mem.is_null() {
            unsafe { CloseHandle(process_handle) };
            return Err(InjectorError::out_of_memory());
        }

        let res = unsafe {
            WriteProcessMemory(
                process_handle,
                path_mem,
                path.as_ptr() as *const _,
                path.len() * 2,
                None,
            )
        }
        .as_bool();

        if !res {
            unsafe { CloseHandle(process_handle) };
            return Err(InjectorError::out_of_memory());
        }

        let kernel32 = unsafe { GetModuleHandleW(w!("kernel32.dll")) }?;
        let load_library = unsafe { GetProcAddress(kernel32, s!("LoadLibraryW")) };

        if load_library.is_none() {
            unsafe { CloseHandle(process_handle) };
            return Err(InjectorError::out_of_memory());
        }

        let thread_handle = unsafe {
            CreateRemoteThread(
                process_handle,
                None,
                0,
                Some(transmute(load_library)),
                Some(path_mem),
                0,
                None,
            )
        }?;

        unsafe {
            CloseHandle(thread_handle);
            CloseHandle(process_handle);
        }

        self.unfreeze()?;

        Ok(())
    }
}
