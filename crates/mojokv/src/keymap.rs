
use std::collections::HashSet;
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

    pub fn get_min_max_ver(&self) -> (u32, u32, HashSet<u32>) {
        let mut set = HashSet::new();
        let (mut min_ver, mut max_ver) = (u32::MAX,0);

        for slot in self.slot_map.iter() {
            if let Some(slot) =  slot {
                for val in slot.iter() {
                    let v = val.get_ver();
                    if v == 0 {
                        break;
                    }
                    set.insert(v);
                    min_ver = min_ver.min(v);
                    max_ver = max_ver.max(v);
                }
            }
        }

        (min_ver, max_ver, set)
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
}