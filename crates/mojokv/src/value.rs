
use modular_bitfield::{bitfield, specifiers::*};
use serde::{Serialize, Deserialize};
use serde::de::Visitor;

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

}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        serializer.serialize_bytes(&self.into_bytes())
    }
}

struct ValueVisitor {}

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {

        if v.len() != 7 {
            return Err(serde::de::Error::invalid_length(v.len(), &self));
        }
        
        Ok(Value::from_bytes([v[0], v[1], v[2], v[3], v[4], v[5], v[6]]))
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("byte array of 7 bytes")
    }

}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        deserializer.deserialize_bytes(ValueVisitor{})
    }
}

/*

pub fn serialize_valuearr<W: std::io::Write>(val_opt: &Option<Vec<Value>>, w: &mut W) -> Result<(), Error> {
    match val_opt {
        Some(val) => {
            let n_items = val.len() as u32;
            w.write_all(&n_items.to_le_bytes())?;

            for elem in val.iter() {
                rmp_serde::encode::write(w, elem)?;
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
            let val = rmp_serde::decode::from_read(r)?;
            tmp_vec.push(val);
        }

        Ok(Some(tmp_vec))
    }

}
*/