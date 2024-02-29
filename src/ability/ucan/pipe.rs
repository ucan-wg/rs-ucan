 use crate::delegation;

pub struct Pipe<
    C: Condition,
    DID: Did,
    V: varsig::Header<Enc>,
    Enc: Codec + TryFrom<u32> + Into<u32>,
        > {
    pub from: delegation::Chain<C, DID, V, Enc>,
    pub to: delegation::Chain<C, DID, V, Enc>,
}
