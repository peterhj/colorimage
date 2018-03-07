/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ffi::gckimg::*;

use byteorder::*;

use std::io::{Read, Seek, Cursor, SeekFrom};

pub fn parse_exif(raw_exif: &[u8]) -> Result<i32, ()> {
  // An APP1 segment larger than 64k violates the JPEG standard.
  if raw_exif.len() > 64 * 1024 {
    return Err(());
  }

  let mut cursor = Cursor::new(raw_exif);
  let mut buf = vec![];

  fn match_bytes(cursor: &mut Cursor<&[u8]>, s: &[u8], buf: &mut Vec<u8>) -> Result<(), ()> {
    buf.clear();
    for _ in 0 .. s.len() {
      buf.push(cursor.read_u8().unwrap());
    }
    if &buf as &[u8] == s {
      Ok(())
    } else {
      Err(())
    }
  }

  if match_bytes(&mut cursor, b"Exif\0\0", &mut buf).is_err() {
    return Err(());
  }

  // Determine byte order.
  if match_bytes(&mut cursor, b"MM\0*", &mut buf).is_ok() {
    _parse_exif_part2::<BigEndian>(&mut cursor, buf)
  } else if match_bytes(&mut cursor, b"II*\0", &mut buf).is_ok() {
    _parse_exif_part2::<LittleEndian>(&mut cursor, buf)
  } else {
    Err(())
  }
}

fn _parse_exif_part2<E>(cursor: &mut Cursor<&[u8]>, mut buf: Vec<u8>) -> Result<i32, ()> where E: ByteOrder {
  // Determine offset of the 0th IFD. (It shouldn't be greater than 64k, which
  // is the maximum size of the entry APP1 segment.)
  let ifd0_offset = match cursor.read_u32::<E>() {
    Ok(x) => x,
    Err(_) => return Err(()),
  };
  if ifd0_offset > 64 * 1024 {
    return Err(());
  }

  // The IFD offset is relative to the beginning of the TIFF header, which
  // begins after the EXIF header, so we need to increase the offset
  // appropriately.
  let ifd0_offset_abs = ifd0_offset + 6;
  match cursor.seek(SeekFrom::Start(ifd0_offset_abs as _)) {
    Ok(_) => {}
    Err(_) => return Err(()),
  };

  let entry_count = match cursor.read_u16::<E>() {
    Ok(x) => x,
    Err(_) => return Err(()),
  };

  for entry in 0 .. entry_count {
    // Read the fields of the entry.
    let tag = match cursor.read_u16::<E>() {
      Ok(x) => x,
      Err(_) => return Err(()),
    };

    // Right now, we only care about orientation, so we immediately skip to the
    // next entry if we find anything else.
    if tag != 0x112 {
      match cursor.seek(SeekFrom::Current(10)) {
        Ok(_) => {}
        Err(_) => return Err(()),
      };
      continue;
    }

    let ty = match cursor.read_u16::<E>() {
      Ok(x) => x,
      Err(_) => return Err(()),
    };

    let count = match cursor.read_u32::<E>() {
      Ok(x) => x,
      Err(_) => return Err(()),
    };

    // Sanity check the type and count.
    if ty != 3 && count != 1 {
      return Err(());
    }

    let value = match cursor.read_u16::<E>() {
      Ok(x) => x,
      Err(_) => return Err(()),
    };

    let exif_orient_code = if value >= 1 && value <= 8 {
      value as i32
    } else {
      return Err(());
    };

    // This is a 32-bit field, but the orientation value only occupies the first
    // 16 bits. We need to advance another 16 bits to consume the entire field.
    match cursor.seek(SeekFrom::Current(2)) {
      Ok(_) => {}
      Err(_) => return Err(()),
    };
    return Ok(exif_orient_code);
  }

  Err(())
}
