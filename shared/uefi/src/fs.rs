use crate::{ EFIStatus, EFIGuid, EFITime };

/// GUID for the SimpleFilesystem protocol
pub const SIMPLE_FILESYSTEM_GUID: EFIGuid =
    EFIGuid {
        data1: 0x964e5b22,
        data2: 0x6459,
        data3: 0x11d2,
        data4: [0x8e, 0x39, 0x0, 0xa0, 0xc9, 0x69, 0x72, 0x3b]
    };

/// GUID for getting file info
pub const GET_INFO_GUID: EFIGuid =
    EFIGuid {
        data1: 0x09576e92,
        data2: 0x6d3f,
        data3: 0x11d2,
        data4: [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b]
    };

/// Struct containing infomation for a file in the SimpleFilesystem
/// NOTE(patrik): This struct is missing the filename because it is behind
/// this struct in memory and the C code uses variable length arrays
/// for structs and rust don't have that feature but there could
/// be an easy fix if we want the filename from the file
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct EFIFileInfo {
    size: u64,
    pub file_size: u64,
    pub physical_size: u64,
    pub create_time: EFITime,
    pub last_access_time: EFITime,
    pub modification_time: EFITime,
    pub attribute: u64,

    // NOTE(patrik): MISSING FILENAME
}

/// An EFI Handle for a directory or file, this struct contains function
/// pointers to muniplulate the directory or file and we create wrappers
/// for those function so we have a better api because all these
/// function pointers are c code so we want a better rust api
#[repr(C)]
pub struct EFIFileHandle {
    revision: u64,

    open_fn: unsafe fn(this: &EFIFileHandle,
                       new_handle: &mut *mut EFIFileHandle,
                       filename: *const u16,
                       open_mode: u64,
                       attributes: u64) -> EFIStatus,
    close_fn: usize,
    delete_fn: usize,
    read_fn: unsafe fn(this: &EFIFileHandle,
                       buffer_size: &mut u64,
                       buffer: *mut u8) -> EFIStatus,
    write_fn: usize,
    get_position_fn: usize,
    set_position_fn: usize,
    get_info_fn: unsafe fn(this: &EFIFileHandle,
                           infomation_type: &EFIGuid,
                           buffer_size: &mut u64,
                           buffer: *mut u8) -> EFIStatus,
    set_info_fn: usize,
    flush_fn: usize,
    open_ex_fn: usize,
    read_ex_fn: usize,
    write_ex_fn: usize,
    flush_ex_fn: usize,
}

impl EFIFileHandle {
    /// Attempt to open a directory or file and return a handle
    /// TODO(patrik): Return a result with a error and remove the panics
    pub fn open<'a>(&self, filename: &str,
                    open_mode: u64, attribute: u64)
        -> &'a EFIFileHandle
    {
        // Create a buffer to construct a UTF-16 filename,
        // used because Rust strings are UTF-8 and uefi only accepts UTF-16
        let mut filename_buffer = [0u16; 1024];

        // Loop through the filename and convert the characters to UTF-16
        let mut index = 0;
        for c in filename.bytes() {
            filename_buffer[index] = c as u16;
            index += 1;
        }

        // Null-Terminate the buffer just to be sure
        filename_buffer[index] = 0u16;

        // Create a null handle ptr
        let mut handle_ptr = core::ptr::null_mut();
        // Try to open the filename
        let status = unsafe {
            (self.open_fn)(self, &mut handle_ptr,
                            filename_buffer.as_ptr(), open_mode, attribute)
        };

        // Check if the status if a success otherwise panic
        if status != EFIStatus::Success {
            // TODO(patrik): Remove this panic
            panic!("'EFIFileHandle' failed to open '{}' error: {:?}",
                   filename, status)
        }

        // Dereference the handle pointer so we get a reference handle
        let handle = unsafe { &*handle_ptr };

        // Return the handle
        handle
    }

    /// Read the file and put all it's content to a buffer
    pub fn read_to_buffer(&self) -> alloc::vec::Vec<u8> {
        // Get the file info
        let file_info = self.get_info();

        // Allocate a buffer for the file content
        let mut buffer = vec![0u8; file_info.file_size as usize];

        // Create a variable for the buffer size
        let mut buffer_size = file_info.file_size;
        // Read the file and put it in the buffer
        let status = unsafe {
            (self.read_fn)(self, &mut buffer_size,
                           buffer.as_mut_ptr())
        };

        // Check the status
        if status != EFIStatus::Success {
            // TODO(patrik): Remove the panic
            panic!("'EFIFileHandle::read_to_buffer' failed to read error: {:?}",
                   status);
        }

        buffer
    }

    // Get the file info
    pub fn get_info(&self) -> EFIFileInfo {
        // Create a variable to retrive the required size for the buffer
        let mut buffer_size = 0u64;

        // Call get info with a null buffer to get the required
        // size for the buffer
        let status = unsafe {
            (self.get_info_fn)(self,
                               &GET_INFO_GUID,
                               &mut buffer_size,
                               core::ptr::null_mut())
        };

        // Check the status becuase we expect it to be a BufferTooSmall error
        if status != EFIStatus::BufferTooSmall {
            // TODO(patrik): Remove the panic
            panic!("'EFIFileHandle::get_info' expected BufferTooSmall got {:?}",
                   status);
        }

        // Allocate a buffer for the info
        let mut buffer = vec![0u8; buffer_size as usize];

        // Get the pointer of the buffer
        let buffer_ptr = buffer.as_mut_ptr();

        // Now, issue the get info with the buffer
        let status = unsafe {
            (self.get_info_fn)(self,
                               &GET_INFO_GUID,
                               &mut buffer_size,
                               buffer_ptr)
        };

        // Check the status
        if status != EFIStatus::Success {
            // TODO(patrik): Remove the panic
            panic!("'EFIFIleHandle::get_info' error: {:?}", status);
        }

        // Cast the buffer pointer to a EFIFileInfo and copy it
        let file_info = unsafe { *(buffer.as_ptr() as *const EFIFileInfo) };

        file_info
    }
}

/// EFISimpleFilesystem is a protocol for filesystem operations
/// i.e reading files
#[repr(C)]
pub struct EFISimpleFilesystem {
    revision: u64,
    open_volume_fn: unsafe fn(this: &EFISimpleFilesystem,
                              root_handle: &mut *mut EFIFileHandle)
                        -> EFIStatus,
}

impl EFISimpleFilesystem {
    /// Open the root volume and return a handle for it
    pub fn open_volume<'a>(&self) -> &'a EFIFileHandle {
        // Create a null handle
        let mut handle_ptr = core::ptr::null_mut();

        // Open the root volume and get the pointer to the handle
        let status = unsafe {
            (self.open_volume_fn)(self, &mut handle_ptr)
        };

        // Check the status
        if status != EFIStatus::Success {
            // TODO(patrik): Remove the panic
            panic!("'EFISimpleFilesystem::open_volume' failed to open root volume - error: {:?}", status);
        }

        // Dereference the pointer and get the reference to the handle
        let handle = unsafe { &*handle_ptr };

        // Return the handle
        handle
    }
}
