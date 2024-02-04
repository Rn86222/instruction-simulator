use crate::linked_fx_hashmap::*;
use crate::{memory::WORD_SIZE, types::*, utils::*};
// use linked_hash_map::LinkedHashMap;

type CacheValue = [MemoryValue; LINE_SIZE / WORD_SIZE];

const CACHE_SIZE: usize = 16 * 1024;
const WAY_NUM: usize = 1;
pub const LINE_SIZE: usize = 16;
const TOTAL_LINE_NUM: usize = CACHE_SIZE / LINE_SIZE;
const LINE_NUM: usize = TOTAL_LINE_NUM / WAY_NUM;

#[derive(Debug, Clone)]
pub struct CacheLine {
    valid: bool,
    dirty: bool,
    accessed: bool,
    tag: Tag,
    value: CacheValue,
}

pub struct Cache {
    values: [LinkedFxHashMap<Tag, CacheLine>; LINE_NUM],
    way_num: usize,
    tag_bit_num: usize,
    index_bit_num: usize,
    offset_bit_num: usize,
}

pub enum CacheAccess {
    HitSet,
    // HitUByte(UByte),
    // HitUHalf(UHalf),
    HitWord(Word),
    Miss,
}

impl Cache {
    pub fn new() -> Self {
        let mut values = vec![];
        for _ in 0..LINE_NUM {
            let mut map = LinkedFxHashMap::new();
            for _ in 0..WAY_NUM {
                map.insert(
                    std::u32::MAX,
                    CacheLine {
                        valid: false,
                        dirty: false,
                        accessed: false,
                        tag: std::u32::MAX,
                        value: [0; LINE_SIZE / WORD_SIZE],
                    },
                );
            }
            values.push(map);
        }
        let values: [LinkedFxHashMap<Tag, CacheLine>; LINE_NUM] = values.try_into().unwrap();
        let way_num = WAY_NUM;
        let line_size = LINE_SIZE;
        let line_num = LINE_NUM;
        let index_bit_num = (line_num as u32).trailing_zeros() as usize;
        let offset_bit_num = (line_size as u32).trailing_zeros() as usize;
        let tag_bit_num = 32 - index_bit_num - offset_bit_num;
        Cache {
            values,
            way_num,
            tag_bit_num,
            index_bit_num,
            offset_bit_num,
        }
    }

    pub fn get_offset_bit_num(&self) -> usize {
        self.offset_bit_num
    }

    fn get_tag(&self, addr: Address) -> Tag {
        addr >> (32 - self.tag_bit_num) as Tag
    }

    fn get_index(&self, addr: Address) -> CacheIndex {
        ((addr << self.tag_bit_num) >> (32 - self.index_bit_num)) as CacheIndex
    }

    fn get_offset(&self, addr: Address) -> usize {
        ((addr << (self.tag_bit_num + self.index_bit_num)) >> (32 - self.offset_bit_num)) as usize
    }

    fn get_status(&self, addr: Address) -> (Tag, CacheIndex, usize) {
        let tag = self.get_tag(addr);
        let index = self.get_index(addr);
        let offset = self.get_offset(addr);
        (tag, index, offset)
    }

    fn update_on_get(cache_line: &mut CacheLine) {
        cache_line.accessed = true;
        cache_line.valid = true;
    }

    fn update_on_set(cache_line: &mut CacheLine) {
        cache_line.dirty = true;
        cache_line.accessed = true;
        cache_line.valid = true;
    }

    // #[allow(dead_code)]
    // pub fn get_ubyte(&mut self, addr: Address) -> CacheAccess {
    //     let (tag, index, offset) = self.get_status(addr);
    //     let cache_line = self.values[index].get_refresh(&tag);
    //     match cache_line {
    //         Some(cache_line) => {
    //             if !cache_line.valid {
    //                 return CacheAccess::Miss;
    //             }
    //             let value = cache_line.value[offset];
    //             Self::update_on_get(cache_line);
    //             CacheAccess::HitUByte(value)
    //         }
    //         None => CacheAccess::Miss,
    //     }
    // }

    // #[allow(dead_code)]
    // pub fn get_uhalf(&mut self, addr: Address) -> CacheAccess {
    //     let (tag, index, offset) = self.get_status(addr);
    //     let cache_line = self.values[index].get_refresh(&tag);
    //     match cache_line {
    //         Some(cache_line) => {
    //             if !cache_line.valid {
    //                 return CacheAccess::Miss;
    //             }
    //             let mut value: UHalf = 0;
    //             for i in 0..2 {
    //                 value += (cache_line.value[offset + i] as UHalf) << (8 * i);
    //             }
    //             Self::update_on_get(cache_line);
    //             CacheAccess::HitUHalf(value)
    //         }
    //         None => CacheAccess::Miss,
    //     }
    // }

    pub fn get_word(&mut self, addr: Address) -> CacheAccess {
        let (tag, index, offset) = self.get_status(addr);
        let cache_line = self.values[index].get_refresh(&tag);
        match cache_line {
            Some(cache_line) => {
                if !cache_line.valid {
                    return CacheAccess::Miss;
                }
                // let mut value: u32 = 0;
                // for i in 0..4 {
                //     value += (cache_line.value[offset + i] as u32) << (8 * i);
                // }
                Self::update_on_get(cache_line);
                let value = cache_line.value[offset >> 2];
                CacheAccess::HitWord(u32_to_i32(value))
            }
            None => CacheAccess::Miss,
        }
    }

    pub fn set_line(
        &mut self,
        addr: Address,
        line: [MemoryValue; LINE_SIZE / WORD_SIZE],
    ) -> Option<[(Address, MemoryValue); LINE_SIZE / WORD_SIZE]> {
        let tag = self.get_tag(addr);
        let index = self.get_index(addr);
        let cache_line_candidates = &self.values[index];
        let cache_line = cache_line_candidates.get(&tag);
        assert!(cache_line.is_none());

        let mut dirty_line_evicted = false;
        let mut evicted_values = [(0, 0); LINE_SIZE / WORD_SIZE];
        if self.values[index].len() >= self.way_num {
            let candidate_for_eviction = self.values[index].pop_front();
            if let Some((_, cache_line)) = candidate_for_eviction {
                if cache_line.dirty {
                    dirty_line_evicted = true;
                    let addr = (cache_line.tag << (self.index_bit_num + self.offset_bit_num))
                        as Address
                        + (index << self.offset_bit_num) as Address;
                    for (i, value) in evicted_values.iter_mut().enumerate() {
                        *value = (addr + i as Address * 4, cache_line.value[i]);
                    }
                }
            }
        }
        let mut cache_line = CacheLine {
            valid: true,
            dirty: false,
            accessed: true,
            tag,
            value: [0; LINE_SIZE / WORD_SIZE],
        };
        cache_line.value[..LINE_SIZE / WORD_SIZE].copy_from_slice(&line[..LINE_SIZE / WORD_SIZE]);
        self.values[index].insert(tag, cache_line);

        if dirty_line_evicted {
            Some(evicted_values)
        } else {
            None
        }
    }

    // #[allow(dead_code)]
    // pub fn set_ubyte(&mut self, addr: Address, value: UByte) -> CacheAccess {
    //     let (tag, index, offset) = self.get_status(addr);
    //     let cache_line = self.values[index].get_refresh(&tag);
    //     match cache_line {
    //         Some(cache_line) => {
    //             if !cache_line.valid {
    //                 return CacheAccess::Miss;
    //             }
    //             cache_line.value[offset] = value;
    //             Self::update_on_set(cache_line);
    //             CacheAccess::HitSet
    //         }
    //         None => CacheAccess::Miss,
    //     }
    // }

    // #[allow(dead_code)]
    // pub fn set_uhalf(&mut self, addr: Address, value: UHalf) -> CacheAccess {
    //     let (tag, index, offset) = self.get_status(addr);
    //     let cache_line = self.values[index].get_refresh(&tag);
    //     match cache_line {
    //         Some(cache_line) => {
    //             if !cache_line.valid {
    //                 return CacheAccess::Miss;
    //             }
    //             for i in 0..2 {
    //                 cache_line.value[offset + i] = ((value >> (i * 8)) & 0xff) as UByte;
    //             }
    //             Self::update_on_set(cache_line);
    //             CacheAccess::HitSet
    //         }
    //         None => CacheAccess::Miss,
    //     }
    // }

    pub fn set_word(&mut self, addr: Address, value: Word) -> CacheAccess {
        let (tag, index, offset) = self.get_status(addr);
        let cache_line = self.values[index].get_refresh(&tag);
        match cache_line {
            Some(cache_line) => {
                let value = i32_to_u32(value);
                if !cache_line.valid {
                    return CacheAccess::Miss;
                }
                // for i in 0..4 {
                //     cache_line.value[offset + i] = ((value >> (i * 8)) & 0xff) as UByte;
                // }
                cache_line.value[offset >> 2] = value;

                Self::update_on_set(cache_line);
                CacheAccess::HitSet
            }
            None => CacheAccess::Miss,
        }
    }
}
