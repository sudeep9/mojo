
use crate::Error;

use modular_bitfield::{bitfield, specifiers::*};

pub type Slot = Option<Vec<Value>>;

#[derive(Clone, Copy)]
#[bitfield]
pub struct Value {
    off: B32,
    ver: B24
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_fmt(format_args!("o={},v={}", self.off(), self.ver()))
    }
}

impl Value {
    pub fn is_allocated(&self) -> bool {
        self.ver() > 0
    }

    pub fn deallocate(&mut self) {
        self.set_off(0);
        self.set_ver(0);
    }

    pub fn put_off(&mut self, off: u32) {
        self.set_off(off);
    }

    pub fn get_off(&self) -> u32 {
        self.off()
    }

    pub fn put_ver(&mut self, v: u32) {
        self.set_ver(v);
    }

    pub fn get_ver(&self) -> u32 {
        self.ver() as u32
    }

    pub fn serialize<W: std::io::Write>(&self, w: &mut W) -> Result<(), Error> {
        let buf = self.into_bytes();
        w.write_all(&buf)?;
        Ok(())
    }

    pub fn deserialize<R: std::io::Read>(r: &mut R) -> Result<Value, Error> {
        let mut buf = [0; std::mem::size_of::<Self>()];
        r.read_exact(&mut buf)?;
        Ok(Value::from_bytes(buf))
    }
}


pub fn serialize_valuearr<W: std::io::Write>(val_opt: &Option<Vec<Value>>, w: &mut W) -> Result<(), Error> {
    match val_opt {
        Some(val) => {
            let n_items = val.len() as u32;
            w.write_all(&n_items.to_le_bytes())?;

            for elem in val.iter() {
                elem.serialize(w)?;
            }
        },
        None => {
            w.write_all(&0u32.to_le_bytes())?;
        }
    }

    Ok(())
}

pub fn deserialize_valuearr<R: std::io::Read>(r: &mut R, pps: usize) -> Result<Option<Vec<Value>>, Error> {
    let mut tmp_buf = [0u8; 4];
    
    r.read_exact(&mut tmp_buf)?;
    let count = u32::from_le_bytes(tmp_buf);

    if count == 0 {
        Ok(None)
    }else{
        if count as usize != pps {
            return Err(Error::UnknownStr("Less number of values than expected".to_owned()));
        }
        let mut tmp_vec = Vec::new();
        for _ in 0..count {
            let val = Value::deserialize(r)?;
            tmp_vec.push(val);
        }

        Ok(Some(tmp_vec))
    }

}