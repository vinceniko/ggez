//! Utility functions.
//!
//! Generally not things end-users have to worry about.

use std::path;
use sdl2::rwops;
use sdl2::surface;
use sdl2_image::ImageRWops;
use std::io::Read;
use std::borrow::Borrow;

use context::Context;
use GameError;
use GameResult;

extern crate owning_ref;

pub fn rwops_from_path<'a>(context: &mut Context,
                           path: &path::Path,
                           buffer: &'a mut Vec<u8>)
                           -> GameResult<rwops::RWops<'a>> {
    let mut stream = try!(context.filesystem.open(path));
    let rw = try!(rwops::RWops::from_read(&mut stream, buffer));
    Ok(rw)
}


pub struct OwningRWops {
    pub rwops: rwops::RWops<'static>,
    buffer: Vec<u8>,
}


impl OwningRWops {
    pub fn new<T: Read+Sized>(stream: &mut T) -> GameResult<OwningRWops> {
        let mut buffer = Vec::new();
        let _ = stream.read_to_end(&mut buffer);
        let rw2;
        {
            let rw = try!(rwops::RWops::from_bytes(&buffer));
            unsafe {
                let rw_ptr = rw.raw();
                rw2 = rwops::RWops::from_ll(rw_ptr);
            }
        }

        Ok(OwningRWops {
            rwops: rw2,
            buffer: buffer,
        })
    }
}

// Here you should just imagine me frothing at the mouth as I
// fight the lifetime checker in circles.
fn clone_surface<'a>(s: surface::Surface<'a>) -> GameResult<surface::Surface<'static>> {
    // let format = pixels::PixelFormatEnum::RGBA8888;
    let format = s.pixel_format();
    // convert() copies the surface anyway, so.
    let res = try!(s.convert(&format));
    Ok(res)
}

/// Loads a given surface.
/// This is here instead of in graphics because it's sorta private-ish
/// (since ggez never exposes a SDL surface directly)
/// but it gets used in context.rs to load and set the window icon.
pub fn load_surface(context: &mut Context,
                    path: &path::Path)
                    -> GameResult<surface::Surface<'static>> {
    let mut buffer: Vec<u8> = Vec::new();
    let rwops = try!(rwops_from_path(context, path, &mut buffer));
    // SDL2_image SNEAKILY adds the load() method to RWops
    // with the ImageRWops trait.
    let surface = try!(rwops.load().map_err(GameError::ResourceLoadError));
    // We *really really* need to clone this surface here because
    // otherwise lifetime interactions between rwops, buffer and surface become
    // intensely painful.
    clone_surface(surface)
}
