#![feature(tuple_trait)]
#![feature(fn_traits)]
#![feature(unboxed_closures)]
#![feature(ptr_metadata)]

use std::{collections::HashMap};

use mmap::{MapOption, MemoryMap};

extern "C" fn print(val: i32) {
    println!("{val}");
}

extern "C" fn bruh() -> i32{
    println!("asd;lasd;klasd;klasd;klasd;klasd");

    1
}



unsafe fn reflect(instructions: &[u8], relocs: DynRelocs, dyn_syms: &HashMap<String, usize>) {
    let map = MemoryMap::new(
        instructions.len(),
        &[
            MapOption::MapReadable,
            MapOption::MapWritable,
            MapOption::MapExecutable,
        ],
    )
    .unwrap();

    std::ptr::copy(instructions.as_ptr(), map.data(), instructions.len());

    let slice = std::slice::from_raw_parts_mut(map.data(), instructions.len());

    for reloc in relocs{
        let addr = *dyn_syms.get(reloc.0).unwrap();
        let offset = reloc.1 as usize;
        let address: &mut [u8; 8] = (&mut slice[offset..(offset + 8)]).try_into().unwrap();
        *address = (addr).to_le_bytes();
    }

    let func: extern "C" fn(i32) = std::mem::transmute(map.data());

    func(10);
}


#[derive(Clone, Copy)]
struct DynRelocs<'a>{
    data: &'a [u8],
    index: usize,
}

impl<'a> DynRelocs<'a>{
    pub fn new(data: &'a[u8]) -> Self{
        Self { data, index: 0 }
    }
}

impl<'a> Iterator for DynRelocs<'a>{
    type Item = (&'a str, u64);

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.len() < self.index + 8{
            self.index = self.data.len();
            return None;
        }
        let pos: [u8; 8] = (&self.data[self.index..(self.index + 8)]).try_into().ok()?;
        let pos = u64::from_be_bytes(pos);
        self.index += 8;

        if self.data.len() < self.index + 2{
            self.index = self.data.len();
            return None;
        }
        let size: [u8; 2] = (&self.data[self.index..(self.index + 2)]).try_into().ok()?;
        let size = u16::from_be_bytes(size) as usize;
        self.index += 2;

        if self.data.len() < self.index + size{
            self.index = self.data.len();
            return None;
        }
        let sym = &self.data[self.index..(self.index + size)];
        let sym = std::str::from_utf8(sym).ok()?;
        self.index += size;

        Some((sym, pos))
    }
}

fn main() {

    let relocs = DynRelocs::new(include_bytes!("../xtra/out/syms"));
    let mut map = HashMap::new();

    map.insert("print".into(), (print as *const()) as usize);
    map.insert("lol_jpg".into(), (bruh as *const()) as usize);

    unsafe { reflect(include_bytes!("../xtra/out/raw.bin"), relocs, &map) }

}
