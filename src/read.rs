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
    ($field_name:ident($size:expr) as String from $bytes:ident[$cursor:ident]) => {
        let $field_name = {
            if $bytes.len() < $cursor + $size as usize {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    format!("Insufficient bytes for {}", stringify!($field_name)),
                ));
            }
            let bytes_array: Vec<u8> = $bytes
                [$cursor..$cursor + $size as usize]
                .try_into()
                .map_err(|_| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Failed to read {}", stringify!($field_name)),
                    )
                })?;
            let value = String::from_utf8(bytes_array).map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to read {}", stringify!($field_name)),
                )
            })?;
            value
        };
        $cursor += $size as usize;
        // Used to make it used in the end
        assert!($cursor == $cursor);
    };
    ($field_name:ident($type:tt) from $bytes:ident[$cursor:ident]) => {
        read!($field_name($type) as $type from $bytes[$cursor]);
    };
    ($field_name:ident($type:tt) as $cast_type:tt from $bytes:ident[$cursor:ident]) => {
        read!($field_name($type) from $bytes[$cursor] | crate::read::byte_count!($type));
        let $field_name = $field_name as $cast_type;
    };
    ($field_name:ident($type:tt) from $bytes:ident[$cursor:ident] | $size:expr) => {
        let $field_name = {
            if $bytes.len() < $cursor + $size as usize {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    format!("Insufficient bytes for {}", stringify!($field_name)),
                ));
            }
            let bytes_array: [u8; $size] = $bytes
                [$cursor..$cursor + $size as usize]
                .try_into()
                .map_err(|_| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Failed to read {}", stringify!($field_name)),
                    )
                })?;
            let value = <$type>::from_le_bytes(bytes_array);
            value
        };
        $cursor += $size as usize;
        // Used to make it used in the end
        assert!($cursor == $cursor);
    };
}

pub(crate) use byte_count;
pub(crate) use read;
