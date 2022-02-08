use std::collections::VecDeque;
use std::path::Path;
use std::thread::current;
use std::time::Duration;
use std::time::Instant;

use linux_embedded_hal::I2cdev;
use linux_embedded_hal::i2cdev::core::I2CDevice;
use linux_embedded_hal::i2cdev::linux::LinuxI2CError;

use std::ffi::CStr;
use std::os::raw::c_char;

const MA_SAMPLE_BUFFER_SIZE: u8 = 50;

pub enum Direction {
    Forward,
    Reverse,
}


pub struct AS5600 {
    context: I2cdev,
    offset: u16,
    direction: Direction,
    samples: VecDeque<(Instant, f32)>, 

}

impl AS5600 {
    /// Opens a new encoder instance using i2c device at `path`
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, LinuxI2CError> {
        let mut i2c = I2cdev::new(path)?;
        i2c.set_slave_address(0x36)?;

        Ok(Self {
            context: i2c,
            offset: 0,
            direction: Direction::Forward,
            samples: VecDeque::with_capacity(MA_SAMPLE_BUFFER_SIZE as usize)
        })
    }

    /// Returns current angle in degrees
    pub fn get_angle(&mut self) -> Result<f32, LinuxI2CError> {
        let bytes = self.context.smbus_read_i2c_block_data(0x0E, 2)?;
        let raw_angle = u16::from_be_bytes([bytes[0], bytes[1]]);

        // Calculate relative angle from zero offset
        let relative_angle: i16 = (((raw_angle - self.offset) + 2048)%4096) as i16 - 2048;

        // Conditionally flip the angle if direction is Reverse
        let flipped = match self.direction {
            Direction::Forward => relative_angle,
            Direction::Reverse => 4096 - relative_angle,
        };

        Ok(flipped as f32/4096. * 360.0)
    }

    /// Sets zero location to current angle
    pub fn zero(&mut self) -> Result<(), LinuxI2CError> {
        let bytes = self.context.smbus_read_i2c_block_data(0x0E, 2)?;
        let raw_angle = u16::from_be_bytes([bytes[0], bytes[1]]);

        self.offset = raw_angle;        
        Ok(())
    }

    /// Sets the direction of the encoder Forward (CW+ / CCW-) or Reverse (CW- / CCW+)
    pub fn set_direction(&mut self, direction: Direction) -> Result<(), LinuxI2CError> {
        self.direction = direction;
        Ok(())
    }

    /// Returns rotational velocity in deg/sec
    pub fn get_velocity(&mut self) -> Result<f32, LinuxI2CError> {
        // If sample buffer is full, pop off the front before pushing the current sample onto the back of the buffer
        if self.samples.len() == MA_SAMPLE_BUFFER_SIZE as usize {
            self.samples.pop_front();
        }
        let current_angle = self.get_angle()?;
        self.samples.push_back((Instant::now(), current_angle));

        // wait and measure until there are two or more samples to calculate velocity
        while self.samples.len() < 2 {
            std::thread::sleep(Duration::from_millis(10));
            let current_angle = self.get_angle()?;
            self.samples.push_back((Instant::now(), current_angle));
        }

        // iterively calculate velocity at each sample timestep
        let mut velocity_buf: VecDeque<f32> = VecDeque::with_capacity(MA_SAMPLE_BUFFER_SIZE as usize);
        let mut iter = self.samples.iter();
   
        let mut prev_sample = iter.next().unwrap();
        for sample in iter {
            let dT = sample.0 - prev_sample.0;
            let dA = sample.1 - prev_sample.1;

            velocity_buf.push_back(dA/dT.as_secs_f32());
            prev_sample = sample
        }

        let sum: f32 = velocity_buf.iter().sum();
        Ok(sum/velocity_buf.len() as f32)
    }
}



// ---------- FFI Functions for python ----------//

#[no_mangle]
/// this function should only be called with a valid i2c device path string
pub unsafe extern "C" fn open_as5600_ffi(path_pointer: *mut c_char) -> *mut AS5600 {
    let bytes = { CStr::from_ptr(path_pointer).to_bytes() };
    let path = std::str::from_utf8(bytes).expect("path should be valid UTF-8");

    if let Ok(ctx) = AS5600::open(path) {
        let boxed = Box::new(ctx);
        Box::into_raw(boxed)
    } else {
        println!("Could not connect to AS5600 at <{path}, 0x36>");
        std::process::exit(-1)
    }
}

#[no_mangle]
/// this function should only be called with a valid AS5600 point from open_as5600_ffi()
pub unsafe extern "C" fn get_angle_ffi(ptr: *mut AS5600) -> f32 {
    let mut encoder = Box::from_raw(ptr);
    let angle = encoder.get_angle().expect("should be able to get angle with valid i2c context");
    Box::leak(encoder);
    angle
    
}

#[no_mangle]
/// this function should only be called with a valid AS5600 pointer from `open_as5600_ffi()` and will block forever
pub unsafe extern "C" fn test_ffi(ptr: *mut AS5600) {
    let mut encoder = Box::from_raw(ptr);
    loop {
        let velocity = encoder.get_velocity().expect("should be able to get velocity with valid i2c context");
        println!("{velocity}");
        std::thread::sleep(Duration::from_secs_f32(1./200.));
    }
}
