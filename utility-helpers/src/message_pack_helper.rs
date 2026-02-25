pub fn serialize_to_message_pack<T: serde::Serialize>(
    data: &T,
) -> Result<Vec<u8>, rmp_serde::encode::Error> {
    rmp_serde::to_vec_named(data)
}

pub fn deserialize_from_message_pack<T: serde::de::DeserializeOwned>(
    data: &[u8],
) -> Result<T, rmp_serde::decode::Error> {
    rmp_serde::from_slice(data)
}
