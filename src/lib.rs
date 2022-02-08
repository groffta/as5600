use std::path::Path;
use std::time::Duration;

use linux_embedded_hal::I2cdev;
use linux_embedded_hal::i2cdev::core::I2CDevice;
use linux_embedded_hal::i2cdev::linux::LinuxI2CError;

use std::ffi::CStr;
use std::os::raw::c_char;

pub struct Context(I2cdev);
impl Context {
    pub fn open<P: AsRef<Path>>(path: P, slave_address: u16) -> Result<Self, LinuxI2CError> {
        let mut i2c = I2cdev::new(path)?;
        i2c.set_slave_address(slave_address)?;

        Ok(Self(i2c))
    }   
}

pub fn get_angle(ctx: &mut Context) -> Result<f64, LinuxI2CError> {
    let bytes = ctx.0.smbus_read_i2c_block_data(0x0E, 2)?;
    let raw_angle = u16::from_be_bytes([bytes[0], bytes[1]]);

    Ok(raw_angle as f64/4096. * 360.0)
}


// ---------- FFI Functions for python ----------//

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn context(path_pointer: *mut c_char) -> *mut Context {
    let bytes = { CStr::from_ptr(path_pointer).to_bytes() };
    let path = std::str::from_utf8(bytes).expect("path should be valid UTF-8");

    if let Ok(ctx) = Context::open(path, 0x36) {
        let boxed = Box::new(ctx);
        Box::into_raw(boxed)
    } else {
        println!("Could not connect to AS5600 at <{path}, 0x36>");
        std::process::exit(-1)
    }
}

#[no_mangle]
/// this function should only be called with a pointer returned by `context()`
pub unsafe extern "C" fn angle(ctx_ptr: *mut Context) -> f64 {
    let mut ctx = Box::from_raw(ctx_ptr);
    let angle = get_angle(&mut ctx).expect("should be able to get angle with valid i2c context");
    Box::leak(ctx);
    angle
    
}

#[no_mangle]
/// this function should only be called with a pointer returned by `context()`
pub unsafe extern "C" fn test(ctx_ptr: *mut Context) {
    let mut ctx = Box::from_raw(ctx_ptr);
    
    loop {
        let angle = get_angle(&mut ctx).expect("should be able to get angle with valid i2c context");
        println!("{angle}");
        std::thread::sleep(Duration::from_secs_f64(1./200.));
    }
}
