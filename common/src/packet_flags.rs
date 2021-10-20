struct ConnectFlags {
    user_name: bool,
    password: bool,
    last_will_retain: bool,
    last_will_qos: bool,
    last_will_flag: bool,
    clean_session: bool,
}

impl ConnectFlags {
    fn new(user_name: bool, password: bool, last_will_retain: bool,
        last_will_qos: bool, last_will_flag: bool, clean_session: bool,
    ) -> ConnectFlags {
        ConnectFlags {
            user_name, password, last_will_retain, last_will_qos, last_will_flag, clean_session
        }
    }
}