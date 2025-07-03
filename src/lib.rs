#![no_std]

extern crate alloc;

mod math;
mod mikktspace;

use core::ffi::{c_float, c_int};
use mikktspace::{genTangSpace, genTangSpaceDefault, SMikkTSpaceContext, SMikkTSpaceInterface};

pub trait MikkTSpaceInterface {
    fn get_num_faces(&self) -> usize;
    fn get_num_vertices_of_face(&self, face: usize) -> usize;
    fn get_position(&self, face: usize, vert: usize) -> [f32; 3];
    fn get_normal(&self, face: usize, vert: usize) -> [f32; 3];
    fn get_tex_coord(&self, face: usize, vert: usize) -> [f32; 2];
    #[expect(unused_variables)]
    fn set_tspace_basic(&mut self, tangent: [f32; 3], sign: f32, face: usize, vert: usize) {}
    #[expect(unused_variables, clippy::too_many_arguments)]
    fn set_tspace(
        &mut self,
        tangent: [f32; 3],
        bi_tangent: [f32; 3],
        mag_s: f32,
        mag_t: f32,
        is_orientation_preserving: bool,
        face: usize,
        vert: usize,
    ) {
    }
}

extern "C" fn get_num_faces_callback(context: *const SMikkTSpaceContext) -> c_int {
    unsafe {
        let interface = &(*((*context).m_pUserData as *const InterfaceWrapper)).interface;
        interface.get_num_faces() as c_int
    }
}

extern "C" fn get_num_vertices_of_face_callback(
    context: *const SMikkTSpaceContext,
    face: c_int,
) -> c_int {
    unsafe {
        let interface = &(*((*context).m_pUserData as *const InterfaceWrapper)).interface;
        interface.get_num_vertices_of_face(face as usize) as c_int
    }
}

extern "C" fn get_position_callback(
    context: *const SMikkTSpaceContext,
    pos_out: *mut c_float,
    face: c_int,
    vert: c_int,
) {
    unsafe {
        let interface = &(*((*context).m_pUserData as *const InterfaceWrapper)).interface;
        let pos = interface.get_position(face as usize, vert as usize);
        *pos_out.offset(0) = pos[0];
        *pos_out.offset(1) = pos[1];
        *pos_out.offset(2) = pos[2];
    }
}

extern "C" fn get_normal_callback(
    context: *const SMikkTSpaceContext,
    norm_out: *mut c_float,
    face: c_int,
    vert: c_int,
) {
    unsafe {
        let interface = &(*((*context).m_pUserData as *const InterfaceWrapper)).interface;
        let normal = interface.get_normal(face as usize, vert as usize);
        *norm_out.offset(0) = normal[0];
        *norm_out.offset(1) = normal[1];
        *norm_out.offset(2) = normal[2];
    }
}

extern "C" fn get_tex_coord_callback(
    context: *const SMikkTSpaceContext,
    texc_out: *mut c_float,
    face: c_int,
    vert: c_int,
) {
    unsafe {
        let interface = &(*((*context).m_pUserData as *const InterfaceWrapper)).interface;
        let tex_coord = interface.get_tex_coord(face as usize, vert as usize);
        *texc_out.offset(0) = tex_coord[0];
        *texc_out.offset(1) = tex_coord[1];
    }
}

extern "C" fn set_tspace_basic_callback(
    context: *const SMikkTSpaceContext,
    tangent: *const c_float,
    sign: c_float,
    face: c_int,
    vert: c_int,
) {
    unsafe {
        let interface = &mut (*((*context).m_pUserData as *mut InterfaceWrapper)).interface;
        let tangent_arr = [*tangent.offset(0), *tangent.offset(1), *tangent.offset(2)];
        interface.set_tspace_basic(tangent_arr, sign, face as usize, vert as usize);
    }
}

extern "C" fn set_tspace_callback(
    context: *const SMikkTSpaceContext,
    tangent: *const c_float,
    bi_tangent: *const c_float,
    mag_s: c_float,
    mag_t: c_float,
    is_orientation_preserving: bool,
    face: c_int,
    vert: c_int,
) {
    unsafe {
        let interface = &mut (*((*context).m_pUserData as *mut InterfaceWrapper)).interface;
        let tangent_arr = [*tangent.offset(0), *tangent.offset(1), *tangent.offset(2)];
        let bi_tangent_arr = [
            *bi_tangent.offset(0),
            *bi_tangent.offset(1),
            *bi_tangent.offset(2),
        ];
        interface.set_tspace(
            tangent_arr,
            bi_tangent_arr,
            mag_s,
            mag_t,
            is_orientation_preserving,
            face as usize,
            vert as usize,
        );
    }
}

const MIKK_INTERFACE: SMikkTSpaceInterface = SMikkTSpaceInterface {
    m_getNumFaces: Some(get_num_faces_callback),
    m_getNumVerticesOfFace: Some(get_num_vertices_of_face_callback),
    m_getPosition: Some(get_position_callback),
    m_getNormal: Some(get_normal_callback),
    m_getTexCoord: Some(get_tex_coord_callback),
    m_setTSpaceBasic: Some(set_tspace_basic_callback),
    m_setTSpace: Some(set_tspace_callback),
};

struct InterfaceWrapper<'a> {
    interface: &'a mut dyn MikkTSpaceInterface,
}

fn create_context(interface_wrapper: &InterfaceWrapper) -> SMikkTSpaceContext {
    SMikkTSpaceContext {
        m_pInterface: &MIKK_INTERFACE as *const _ as *mut _,
        m_pUserData: interface_wrapper as *const _ as *mut _,
    }
}

pub fn gen_tang_space_default<I>(interface: &mut I) -> bool
where
    I: MikkTSpaceInterface,
{
    let interface_wrapper = InterfaceWrapper { interface };
    let context = create_context(&interface_wrapper);
    unsafe { genTangSpaceDefault(&context) }
}

pub fn gen_tang_space<I>(interface: &mut I, angular_threshold: f32) -> bool
where
    I: MikkTSpaceInterface,
{
    let interface_wrapper = InterfaceWrapper { interface };
    let context = create_context(&interface_wrapper);
    unsafe { genTangSpace(&context, angular_threshold) }
}
