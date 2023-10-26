use crate::entity::{Unk80809c36, Unk80809c0f};
use crate::map::Unk808099d6;
use crate::structure::{TablePointer, Tag, RelPointer};
use crate::types::DestinyHash;

use binrw::{BinRead, NullString};
use destiny_pkg::TagHash;
use std::io::SeekFrom;


#[derive(BinRead, Debug)]
pub struct Unk80809994 {
    pub file_size: u64,
    pub unk8: DestinyHash,
    // #[br(seek_before(SeekFrom::Start(0x28)))]
    // pub unk28: Tag<Unk80809a88>,
    #[br(seek_before(SeekFrom::Start(0x30)))]
    pub unk30: RelPointer<NullString>,
    // pub unk38: Tag<Unk80809780>,
    #[br(seek_before(SeekFrom::Start(0x50)))]
    // pub unk40: TablePointer<Unk8080924a>,
    pub unk50: TablePointer<Unk8080924d>,
    // pub unk60: TablePointer<Unk8080924b>,
}

// #[derive(BinRead, Debug)]
// pub struct Unk80809780 {
//     pub file_size: u64,
//     pub unk8: TablePointer<Unk80809252>,
// }

// #[derive(BinRead, Debug)]
// pub struct Unk8080924a {
//     pub unk0: u64,
//     pub unk8: RelPointer<NullString>,
// }

#[derive(BinRead, Debug)]
pub struct Unk8080924d {
    pub unk0: u64,
    pub unk8: TablePointer<Unk8080924f>,
    // pub t1: u64,
    // pub t2: u64,
}

#[derive(BinRead, Debug)]
pub struct Unk8080924f {
    pub unk0: u32,
    pub unk4: u32,
    #[br(seek_before(SeekFrom::Current(0x3C)))]
    pub unk44: Tag<Unk8080925b>, // Unk8080925b
    // pub unk44: TagHash,
    pub unk48: u32,
}


// #[derive(BinRead, Debug)]
// pub struct Unk80809252 {
//     pub unk0: u32,
//     pub unk4: Tag<Unk80809c0f>
// }


#[derive(BinRead, Debug)]
pub struct Unk8080925b {
    pub file_size: u64,
    pub unk8: RelPointer<NullString>,
    pub unk10: u32,
    pub unk14: Tag<Unk8080925e>,
}

#[derive(BinRead, Debug)]
pub struct Unk8080925e {
    pub file_size: u64,
    pub unk8: TablePointer<Unk80809260>,
    // another table pointer @ 0x18 but not sure what
    #[br(seek_before(SeekFrom::Start(0x28)))]
    pub unk28: TablePointer<Unk80809260>,
}

#[derive(BinRead, Debug)]
pub struct Unk80809260 {
    pub unk0: Tag<Unk80809462>
}

#[derive(BinRead, Debug)]
pub struct Unk80809462 {
    pub file_size: u64,
    pub unk8: DestinyHash,
    pub unkc: DestinyHash,
    // something else between there and here
    #[br(seek_before(SeekFrom::Start(0x20)))]
    pub unk20: TablePointer<Unk80809b3a>, // dunno
    pub unk30: u64, // string index + hash?
    pub unk38: TablePointer<Unk80809464>, // entities n stuff?
}

#[derive(BinRead, Debug)]
pub struct Unk80809464 {
    pub unk0: u32,
    pub unk4: DestinyHash,
    pub unk8: TablePointer<Tag<Unk80809468>>, // Unk80809468
}

// #[derive(BinRead, Debug)]
// pub struct Unk80809466 {
//     pub unk0: Tag<Unk80809468>,
// }

#[derive(BinRead, Debug)]
pub struct Unk80809b3a {
    // string index + hash?
    pub unk0: u32,
    pub unk4: u32,
}


#[derive(BinRead, Debug)]
pub struct Unk80809468 {
    pub file_size: u64,
    pub unk8: TagHash,
    #[br(seek_before(SeekFrom::Start(0x10)))]
    pub unk10: TablePointer<Tag<Unk80809b14>>, // Unk80809b13
}

// #[derive(BinRead, Debug)]
// pub struct Unk80809b13 {
//     pub unk0: Tag<Unk80809b14>,
// }

#[derive(BinRead, Debug)]
pub struct Unk80809b14 {
    pub file_size: u64,
    pub unk8: DestinyHash,
    pub unkc: Tag<Unk80809c36>,
    pub unk10: Tag<Unk80809c36>,
    // 27998080 @ 0x14
}