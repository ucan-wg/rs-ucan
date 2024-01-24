use libipld_core::ipld::Ipld;

pub struct FieldName {
    name: Box<str>,
}

pub enum Condition {
    Contains { field: FieldName, value: Vec<Ipld> },
    MinLength { field: FieldName, value: u64 },
    MaxLength { field: FieldName, value: u64 },
    Equals { field: FieldName, value: Ipld },
    Regex { field: FieldName }, // FIXME

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
