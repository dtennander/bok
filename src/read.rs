macro_rules! byte_count {
    (u8) => {
        1
    };
    (i32) => {
        4
    };
    (i64) => {
        8
    };
    (u32) => {
        4
    };
    (u64) => {
        8
    };
}

macro_rules! read {
    ($field_name:ident($size:expr) as String from $reader:ident) => {
        let $field_name = {
            let mut byte_array = vec![0; $size];
            $reader.read_exact(&mut byte_array).map_err(|_|
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to read {}", stringify!($field_name)),
                ))?;
            let value = String::from_utf8(byte_array).map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to read {}", stringify!($field_name)),
                )
            })?;
            value
        };
    };
    ($field_name:ident($type:tt) from $reader:ident using $buffer:ident) => {
        read!($field_name($type) as $type from $reader using $buffer)
    };
    ($field_name:ident($type:tt) as $cast_type:tt from $reader:ident using $buffer:ident) => {
        read!($field_name($type) from $reader using $buffer | crate::read::byte_count!($type));
        let $field_name = $field_name as $cast_type;
    };
    ($field_name:ident($type:tt) from $reader:ident using $buffer:ident | $size:expr) => {
        let $field_name = {
            let mut byte_array: [u8; $size] =
                $buffer[..$size]
                 .try_into()
                 .map_err(|_| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Buffer to small",
                    )
                })?;
            $reader.read_exact(&mut byte_array).map_err(|_|
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to read {}", stringify!($field_name)),
                ))?;
            let value = <$type>::from_le_bytes(byte_array);
            value
        };
    };
}

pub(crate) use byte_count;
pub(crate) use read;
