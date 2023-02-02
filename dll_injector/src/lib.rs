use std::mem::{size_of, transmute};

use error::InjectorError;
use windows::{
    s, w,
    Win32::{
        Foundation::{CloseHandle, HANDLE},
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
            Memory::{VirtualAllocEx, MEM_COMMIT, PAGE_READWRITE},
            Threading::{
                CreateRemoteThread, OpenProcess, OpenThread, ResumeThread, SuspendThread,
                PROCESS_CREATE_THREAD, PROCESS_VM_OPERATION, PROCESS_VM_WRITE, THREAD_ALL_ACCESS,
            },
        },
    },
};

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
        let mut process = None;
        while process.is_none() {
            process = Process::find_process(starts_with)?;
        }

        Ok(process.unwrap())
    }

    fn foreach_thread(&self, cb: fn(HANDLE)) -> Result<(), InjectorError> {
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

                cb(thread);
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

    pub fn inject_dll(&self, dll_path: &str) -> Result<(), InjectorError> {
        self.freeze()?;

        let process_handle = unsafe {
            OpenProcess(
                PROCESS_CREATE_THREAD | PROCESS_VM_OPERATION | PROCESS_VM_WRITE,
                false,
                self.pid,
            )
        }?;

        let path = dll_path.encode_utf16().collect::<Vec<u16>>();
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
