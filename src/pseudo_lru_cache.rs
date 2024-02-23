use crate::{memory::WORD_SIZE, types::*, utils::*};

type CacheValue = [MemoryValue; LINE_SIZE / WORD_SIZE];

const CACHE_SIZE: usize = 16 * 4 * 2048;
const WAY_NUM: usize = 4;
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

pub struct PseudoLRUCache {
    values: [[CacheLine; WAY_NUM]; LINE_NUM],
    tag_bit_num: usize,
    index_bit_num: usize,
    offset_bit_num: usize,
}

pub enum CacheAccess {
    HitSet,
    HitWord(Word),
    Miss,
}

impl PseudoLRUCache {
    pub fn new() -> Self {
        let mut values: Vec<[CacheLine; WAY_NUM]> = vec![];
        for _ in 0..LINE_NUM {
            let mut line = vec![];
            for _ in 0..WAY_NUM {
                line.push(CacheLine {
                    valid: false,
                    dirty: false,
                    accessed: false,
                    tag: std::u32::MAX,
                    value: [0; LINE_SIZE / WORD_SIZE],
                });
            }
            values.push(line.try_into().unwrap());
        }
        let values: [[CacheLine; WAY_NUM]; LINE_NUM] = values.try_into().unwrap();
        let line_size = LINE_SIZE;
        let line_num = LINE_NUM;
        let index_bit_num = (line_num as u32).trailing_zeros() as usize;
        let offset_bit_num = (line_size as u32).trailing_zeros() as usize;
        let tag_bit_num = 32 - index_bit_num - offset_bit_num;
        PseudoLRUCache {
            values,
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

    fn get_cache_line(&self, index: usize, tag: Tag) -> Option<&CacheLine> {
        self.values[index].iter().find(|&line| line.tag == tag)
    }

    fn get_cache_line_refresh(&mut self, index: usize, tag: Tag) -> Option<&mut CacheLine> {
        let mut found = false;
        for line in self.values[index].iter() {
            if line.tag == tag {
                found = true;
                break;
            }
        }
        if found {
            let mut all_accessed = true;
            for line in self.values[index].iter() {
                if line.tag != tag && !line.accessed {
                    all_accessed = false;
                    break;
                }
            }
            if all_accessed {
                for line in self.values[index].iter_mut() {
                    if line.tag != tag {
                        line.accessed = false;
                    }
                }
            }
            for line in self.values[index].iter_mut() {
                if line.tag == tag {
                    line.accessed = true;
                    return Some(line);
                }
            }
            unreachable!();
        }
        None
    }

    fn update_on_get(cache_line: &mut CacheLine) {
        cache_line.valid = true;
    }

    fn update_on_set(cache_line: &mut CacheLine) {
        cache_line.dirty = true;
        cache_line.valid = true;
    }

    pub fn get_word(&mut self, addr: Address) -> CacheAccess {
        let (tag, index, offset) = self.get_status(addr);
        let cache_line = self.get_cache_line_refresh(index, tag);
        match cache_line {
            Some(cache_line) => {
                if !cache_line.valid {
                    return CacheAccess::Miss;
                }
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
        let cache_line = self.get_cache_line(index, tag);
        assert!(cache_line.is_none());

        let mut new_line = CacheLine {
            valid: true,
            dirty: false,
            accessed: true,
            tag,
            value: [0; LINE_SIZE / WORD_SIZE],
        };
        new_line.value[..LINE_SIZE / WORD_SIZE].copy_from_slice(&line[..LINE_SIZE / WORD_SIZE]);

        let mut dirty_line_evicted = false;
        let mut evicted_values = [(0, 0); LINE_SIZE / WORD_SIZE];

        let mut will_evict = true;
        for target_line in self.values[index].iter_mut() {
            if !target_line.valid {
                will_evict = false;
                *target_line = new_line.clone();
                break;
            }
        }

        if will_evict {
            let mut way_index = WAY_NUM + 1;
            for (i, candidate) in self.values[index].iter_mut().enumerate() {
                if !candidate.accessed {
                    way_index = i;
                    if candidate.dirty {
                        dirty_line_evicted = true;
                        let addr = (candidate.tag << (self.index_bit_num + self.offset_bit_num))
                            as Address
                            + (index << self.offset_bit_num) as Address;
                        for (i, value) in evicted_values.iter_mut().enumerate() {
                            *value = (addr + i as Address * 4, candidate.value[i]);
                        }
                    }
                    break;
                }
            }
            self.values[index][way_index] = new_line;
        }

        let mut all_accessed = true;
        for line in self.values[index].iter() {
            if !line.accessed {
                all_accessed = false;
                break;
            }
        }
        if all_accessed {
            for line in self.values[index].iter_mut() {
                if line.tag != tag {
                    line.accessed = false;
                }
            }
        }

        if dirty_line_evicted {
            Some(evicted_values)
        } else {
            None
        }
    }

    pub fn set_word(&mut self, addr: Address, value: Word) -> CacheAccess {
        let (tag, index, offset) = self.get_status(addr);
        let cache_line = self.get_cache_line_refresh(index, tag);
        match cache_line {
            Some(cache_line) => {
                let value = i32_to_u32(value);
                if !cache_line.valid {
                    return CacheAccess::Miss;
                }
                cache_line.value[offset >> 2] = value;

                Self::update_on_set(cache_line);
                CacheAccess::HitSet
            }
            None => CacheAccess::Miss,
        }
    }
}
