// SPDX-License-Identifier: GPL-2.0

//! Rust driver completion

// use core::ptr;
use core::result::Result::{Err, Ok};

use kernel::prelude::*;
// use kernel::sync::Mutex;
use kernel::{chrdev, file};

use kernel::task::Task as task;

use kernel::bindings;
// use core::ffi::c_void;
// const GLOBALMEM_SIZE: usize = 0x1000;

module! {
    type: RustChrdev,
    name: "rust_completion",
    author: "WoodHolz",
    description: "Rust completion",
    license: "GPL",
}

// static GLOBALMEM_BUF: Mutex<[u8;GLOBALMEM_SIZE]M_SIZE]afe {
//     Mutex::new([0u8;GLOBALMEM_SIZE])
// };

// struct RustFile {
//     #[allow(dead_code)]
//     inner: &'static Mutex<[u8; COMPLETION]>,
// }
static mut GLOBALMEM_COMP: Option<bindings::completion> = None;
struct RustCompletion {
}

#[vtable]
impl file::Operations for RustCompletion {
    type Data = Box<Self>;

    fn open(_shared: &(), _file: &file::File) -> Result<Box<Self>> {
        pr_info!("completion_open is invoked\n");
        Ok(
            Box::try_new(RustCompletion {})?
        )
    }

    fn write(
        _this: &Self,
        _file: &file::File,
        _reader: &mut impl kernel::io_buffer::IoBufferReader,
        _offset:u64,
    ) -> Result<usize> {
        
        pr_info!("completion_write is invoked\n");
        pr_info!("process {} awakening the readers...\n", task::current().pid());
        if _reader.is_empty() {
            return Ok(0);
        }
        unsafe {
            if let Some(ref mut completion) = &mut GLOBALMEM_COMP {
                let completion = completion as *mut bindings::completion;
                bindings::complete(completion);
            }
        }
        Ok(_reader.len())

        // let mut contents = _this.inner.lock(); // 加锁 互斥 connents
        // // // _this.inner.lock();
        // let len = _reader.len();
        // _reader.read_slice(& mut contents[0..len])?;
        // // // _reader.read_all()?;
        // pr_info!("write !!!!\n");
        // Ok(len)

        // /* 2 */
        // let mut buffer = _this.inner.lock();
        // let len = _reader.len();
        // let reader_ptr = _reader as *mut dyn kernel::io_buffer::IoBufferReader as *const c_void;
        // unsafe { 
        //     bindings::_copy_from_user(buffer.as_mut_ptr(), reader_ptr, len as u64)
        // }

        // Err(EPERM)
        
    }

    fn read(_this: &Self, _file: &file::File, _writer: &mut impl kernel::io_buffer::IoBufferWriter, _offset:u64,) -> Result<usize> {

        pr_info!("completion_read is invoked\n");
        pr_info!("process {} awakening the readers...\n", task::current().pid());

        unsafe {
            if let Some(ref mut completion) = &mut GLOBALMEM_COMP {
                let completion = completion as *mut bindings::completion;
                bindings::wait_for_completion(completion);
            }
        }
        pr_info!("awoken {}\n", task::current().pid());
        Ok(_writer.len())

        // let contents = _this.inner.lock(); // 加锁 互斥 connents
        // let data = & contents[_offset as usize ..];
        // _writer.write_slice(data)?;
        // _writer.write_slice(& contents[_offset as usize ..])?;
        // unsafe {
        //     _writer.write_raw(data.as_ptr() , data.len())?;
        // }
        // Ok(data.len())
        // Err(EPERM)
    }
}

struct RustChrdev {
    _dev: Pin<Box<chrdev::Registration<2>>>,
}

impl kernel::Module for RustChrdev {
    fn init(name: &'static CStr, module: &'static ThisModule) -> Result<Self> {
        unsafe {
            GLOBALMEM_COMP = Some(bindings::completion::default());
            if let Some(ref mut completion) = &mut GLOBALMEM_COMP {
                let completion = completion as *mut bindings::completion;
                bindings::init_completion(completion);
            }
        }


        pr_info!("Rust character device sample (init)\n");
           
        let mut chrdev_reg = chrdev::Registration::new_pinned(name, 0, module)?;

        // Register the same kind of device twice, we're just demonstrating
        // that you can use multiple minors. There are two minors in this case
        // because its type is `chrdev::Registration<2>`
        chrdev_reg.as_mut().register::<RustCompletion>()?;
        chrdev_reg.as_mut().register::<RustCompletion>()?;

        Ok(RustChrdev { _dev: chrdev_reg })
    }
}

impl Drop for RustChrdev {
    fn drop(&mut self) {
        pr_info!("Rust character device sample (exit)\n");
    }
}
