
use crate::value::{Value, Slot};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct KeyMap {
    pub slot_map: Vec<Slot>,
    pps: usize
}

impl KeyMap {
    //TODO: remove this flag
    pub fn new(pps: usize) -> Self {
        KeyMap { 
            slot_map: Vec::new(),
            pps,
        }
    }

    fn alloc_value_arr(pps: usize) -> Vec<Value> {
        let v = vec![Value::new(); pps];
        v
    }

    pub fn put(&mut self, key: u32, val: Value) {
        let slot = (key as usize)/self.pps;
        if slot >= self.slot_map.len() {
            log::debug!("KeyMap put key={}, value= {:?} slot={} kmaplen={}", key, val, slot, self.slot_map.len());
            for _ in 0..(slot - self.slot_map.len() + 1) {
                self.slot_map.push(None);
            }
        }

        let val_arr = self.slot_map[slot].get_or_insert_with(||{
            KeyMap::alloc_value_arr(self.pps)
        });

        let slot_key = key % (self.pps as u32);
        log::debug!("KeyMap put slot_key={}", slot_key);
        val_arr[slot_key as usize] = val; 
    }

    pub fn get(&self, key: u32) -> Option<&Value> {
        let slot = key/self.pps as u32;
        log::debug!("KeyMap get key={}, slot={}, kmaplen={}", key, slot, self.slot_map.len());
        if slot as usize >= self.slot_map.len() {
            return None;
        }

        self.slot_map[slot as usize].as_ref().map(|val_arr|{
            let slot_key = key % self.pps as u32;
            &val_arr[slot_key as usize]
        })
    }

    pub fn truncate(&mut self, key: u32) {
        let slot = key/self.pps as u32;
        self.slot_map.truncate((slot+1) as usize);
        let slot_key = key % (self.pps as u32);

        if let Some(slot_vec) = self.slot_map[slot as usize].as_mut() {
            for i in slot_key as usize ..slot_vec.len() {
                slot_vec[i].deallocate();
            }
        }
    }

    /*
    pub fn serialize<W: std::io::Write>(&self, w: &mut W) -> Result<(), Error> {
        w.write_all(&(self.pps as u32).to_le_bytes())?;

        let tmp_buf = (self.slot_map.len() as u32).to_le_bytes();
        w.write_all(&tmp_buf)?;


        for slot in self.slot_map.iter() {
            log::trace!("serialize slot: {:?}", slot);
            serialize_valuearr(slot, w)?;
        }

        Ok(())
    }

    pub fn deserialize<R: std::io::Read>(r: &mut R) -> Result<KeyMap, Error> {
        log::debug!("deserialize keymap");

        let pps = utils::read_le_u32(r)?;
        log::debug!("deserializing keymap pps={}", pps);

        let nslots = utils::read_le_u32(r)?;
        log::debug!("deserializing slots={}", nslots);

        let mut kmap = KeyMap::new(pps as usize);
        for _ in 0..nslots {
            let slot = deserialize_valuearr(r, pps as usize)?;
            kmap.slot_map.push(slot);
        }

        Ok(kmap)
    }
 */
}