pub struct ConnectFlags {
    username: bool,
    password: bool,
    last_will_retain: bool,
    last_will_qos: bool,
    last_will_flag: bool,
    clean_session: bool,
}

impl ConnectFlags {
    pub fn new(
        username: bool,
        password: bool,
        last_will_retain: bool,
        last_will_qos: bool,
        last_will_flag: bool,
        clean_session: bool,
    ) -> ConnectFlags {
        ConnectFlags {
            username,
            password,
            last_will_retain,
            last_will_qos,
            last_will_flag,
            clean_session,
        }
    }
    pub fn set_username(&mut self, with_username: bool) {
        self.username = with_username;
    }
    pub fn set_password(&mut self, with_password: bool) {
        self.password = with_password;
    }
    pub fn set_last_will_retain(&mut self, last_will_retain: bool) {
        self.last_will_retain = last_will_retain;
    }
    pub fn set_last_will_qos(&mut self, last_will_qos: bool) {
        self.last_will_qos = last_will_qos;
    }
    pub fn set_last_will_flag(&mut self, last_will_flag: bool) {
        self.last_will_flag = last_will_flag;
    }
    pub fn set_clean_session(&mut self, clean_session: bool) {
        self.clean_session = clean_session;
    }
}
