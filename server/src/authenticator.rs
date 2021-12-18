use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, ErrorKind};

pub struct Authenticator {
    accounts: HashMap<String, String>,
}

impl Authenticator {
    fn new(accounts: HashMap<String, String>) -> Authenticator {
        Authenticator { accounts }
    }

    pub fn from(filename: String) -> Result<Authenticator, Error> {
        let mut hash: HashMap<String, String> = HashMap::new();
        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.unwrap();
            let vec: Vec<&str> = line.split(";").collect();
            if vec.len() != 2 {
                return Err(Error::new(ErrorKind::Other, "Incorrect format"));
            }

            if hash.contains_key(&vec.get(0).unwrap().to_string()) {
                return Err(Error::new(ErrorKind::Other, "Username already in use"));
            }
            hash.insert(
                vec.get(0).unwrap().to_string(),
                vec.get(1).unwrap().to_string(),
            );
        }

        Ok(Authenticator::new(hash))
    }

    pub fn account_is_valid(&self, username: &String, password: &String) -> bool {
        match self.accounts.get(username) {
            None => false,
            Some(pass) => pass == password,
        }
    }
}

/* ------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Read, Write};

    #[test]
    fn valid_account_returns_true() {
        let mut hash: HashMap<String, String> = HashMap::new();
        hash.insert("usuario1".to_string(), "contraseña1".to_string());
        hash.insert("usuario2".to_string(), "contraseña2".to_string());
        hash.insert("usuario3".to_string(), "contraseña3".to_string());
        let authenticator = Authenticator::new(hash);
        let to_test =
            authenticator.account_is_valid(&"usuario2".to_string(), &"contraseña2".to_string());
        assert_eq!(to_test, true);
    }

    #[test]
    fn username_not_existing_returns_false() {
        let mut hash: HashMap<String, String> = HashMap::new();
        hash.insert("usuario1".to_string(), "contraseña1".to_string());
        hash.insert("usuario2".to_string(), "contraseña2".to_string());
        hash.insert("usuario3".to_string(), "contraseña3".to_string());
        let authenticator = Authenticator::new(hash);
        let to_test =
            authenticator.account_is_valid(&"usuario4".to_string(), &"contraseña4".to_string());
        assert_eq!(to_test, false);
    }

    #[test]
    fn incorrect_password_returns_false() {
        let mut hash: HashMap<String, String> = HashMap::new();
        hash.insert("usuario1".to_string(), "contraseña1".to_string());
        hash.insert("usuario2".to_string(), "contraseña2".to_string());
        hash.insert("usuario3".to_string(), "contraseña3".to_string());
        let authenticator = Authenticator::new(hash);
        let to_test =
            authenticator.account_is_valid(&"usuario2".to_string(), &"contraseña3".to_string());
        assert_eq!(to_test, false);
    }
}
