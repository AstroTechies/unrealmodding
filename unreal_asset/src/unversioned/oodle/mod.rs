#[cfg(feature = "oodle")]
// #[cfg(all(target_os = "win32", target_arch = "x64"))]
#[link(name = "oo2core_9_win64")]
#[allow(non_snake_case)]
extern "C" {
    pub fn OodleLZ_Decompress(
        buffer: *const u8,
        buffer_size: u64,
        output_buffer: *mut u8,
        output_buffer_size: u64,
        a: u32,
        b: u32,
        c: u32,
        d: u32,
        e: u32,
        f: u32,
        g: u32,
        h: u32,
        i: u32,
        thread_module: u32,
    ) -> i32;
}

#[cfg(feature = "oodle")]
pub fn decompress(buffer: &[u8], size: u64, uncompressed_size: u64) -> Option<Vec<u8>> {
    let mut decompressed_buffer = Vec::with_capacity(uncompressed_size as usize);
    let decompressed_count = unsafe {
        OodleLZ_Decompress(
            buffer.as_ptr(),
            size,
            decompressed_buffer.as_mut_ptr(),
            uncompressed_size,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            3,
        )
    };

    if decompressed_count == 0 {
        return None;
    }

    decompressed_buffer.resize(decompressed_count as usize, 0);
    return Some(decompressed_buffer);
}
