/*
Unreal pak format
File parts:
    - recordss
        - header
        - data
    - index
        - records entries
            - record name
            - record header
    - footer

header:
    - u64 offset (when infront of record empty)
    - u64 size
    - u64 size decompressed
    - u32 compression method
    - 20 bytes sha1 hash
    - compression block data (only when compression method is not 0)
        - u32 number of blocks
        - blocks
            - u64 block start
            - u64 block end
    - u8 is encrypted flag
    - u32 block size
*/
