pub trait Tagged {
    type Tag;
}

pub trait VariantOf<Enum: Tagged>: Into<Enum> + TryFrom<Enum, Error = Enum> {
    const TAG: Enum::Tag;
}
