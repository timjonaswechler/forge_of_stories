use slotmap::new_key_type;

new_key_type! { pub struct NotificationKey; }

pub enum NotificationKind {
    Info,
    Success,
    Error,
    Progress(Option<u8>),
}

pub struct Notification {
    pub id: NotificationKey,
    pub kind: NotificationKind,
    pub message: String,
    pub created_at: std::time::Instant,
    pub ttl: std::time::Duration,
}
