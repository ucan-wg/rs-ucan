use libipld_core::ipld::Ipld;

pub enum Condition {
    Contains { field: String, value: Vec<Ipld> },
    MinLength { field: String, value: u64 },
    MaxLength { field: String, value: u64 },
    Equals { field: String, value: Ipld },
    Regex { field: String }, // FIXME

    // Silly example
    OnDayOfWeek { day: Day },
}

pub enum Day {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}
