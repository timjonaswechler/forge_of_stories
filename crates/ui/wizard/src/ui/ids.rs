// ui/ids.rs
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct PageId(pub u16);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct PopupId(pub u16);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct CompId(pub u32);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct TaskId(pub u64);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct NotifId(pub u64);
