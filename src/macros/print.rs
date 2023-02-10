#[macro_export]
macro_rules! pretty_print_hex_field {
    ($name:ident, $value:expr) => {
        print!(
            "{:min_width$}:  {:#04x?}\n",
            stringify!($name),
            $value,
            min_width = 20
        );
    };
}
#[macro_export]
macro_rules! pretty_print_var_hex {
    ($var_str:expr, $value:expr) => {
        print!(
            "{:min_width$}:  {:#04x?}\n",
            $var_str,
            $value,
            min_width = 20
        );
    };
}

#[macro_export]
macro_rules! pretty_print_hex_fields {
    ($self:ident, $( $i:ident ),+) => {
        $(
            pretty_print_hex_field!($i, $self.$i.to_le_bytes()[0]);
        )+
        println!();
    };
}

#[macro_export]
macro_rules! pretty_print_name_hex_fields {
    ($type:ty, $self:ident, $( $i:ident ),+) => {
        print!("{}: 0x", stringify!($type));
        $(
            print!("{:02x}", $self.$i.to_le_bytes()[0]);
        )+
        println!();
        $(
            print!("{:ident$}", "", ident = 2);
            pretty_print_hex_field!($i, $self.$i.to_le_bytes()[0]);
        )+
        println!();
    };
}
