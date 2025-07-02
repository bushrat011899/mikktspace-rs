use core::ops::{Add, Sub};

use crate::libc::{c_double, c_float, c_int};

pub const M_PI: c_double = 3.141_592_653_589_793;
pub const __FLT_MIN__: c_float = 1.175_494_4e-38;
pub const FLT_MIN: c_float = __FLT_MIN__;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SVec3 {
    pub x: c_float,
    pub y: c_float,
    pub z: c_float,
}

impl Add for SVec3 {
    type Output = SVec3;

    fn add(self, rhs: Self) -> Self::Output {
        SVec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for SVec3 {
    type Output = SVec3;

    fn sub(self, rhs: Self) -> Self::Output {
        SVec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

pub fn acos(x: c_double) -> c_double {
    unsafe {
        extern "C" {
            fn acos(_: c_double) -> c_double;
        }
        acos(x)
    }
}

pub fn cos(x: c_double) -> c_double {
    unsafe {
        extern "C" {
            fn cos(_: c_double) -> c_double;
        }
        cos(x)
    }
}

fn sqrtf(x: c_float) -> c_float {
    unsafe {
        extern "C" {
            fn sqrtf(_: c_float) -> c_float;
        }
        sqrtf(x)
    }
}

pub fn fabsf(x: c_float) -> c_float {
    unsafe {
        extern "C" {
            fn fabsf(_: c_float) -> c_float;
        }
        fabsf(x)
    }
}

pub fn veq(v1: SVec3, v2: SVec3) -> c_int {
    (v1.x == v2.x && v1.y == v2.y && v1.z == v2.z) as c_int
}

pub fn vscale(scale: c_float, v: SVec3) -> SVec3 {
    SVec3 {
        x: scale * v.x,
        y: scale * v.y,
        z: scale * v.z,
    }
}

pub fn length_squared(v: SVec3) -> c_float {
    v.x * v.x + v.y * v.y + v.z * v.z
}

pub fn length(v: SVec3) -> c_float {
    sqrtf(length_squared(v))
}

pub fn normalize(v: SVec3) -> SVec3 {
    vscale(1 as c_int as c_float / length(v), v)
}

pub fn vdot(v1: SVec3, v2: SVec3) -> c_float {
    v1.x * v2.x + v1.y * v2.y + v1.z * v2.z
}

pub fn not_zero(x: c_float) -> c_int {
    (fabsf(x) > FLT_MIN) as c_int
}

pub fn v_not_zero(v: SVec3) -> c_int {
    (not_zero(v.x) != 0 || not_zero(v.y) != 0 || not_zero(v.z) != 0) as c_int
}
