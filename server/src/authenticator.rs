use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, ErrorKind};

pub struct Authenticator{
    accounts: HashMap<String, String>
}

impl Authenticator {
    pub fn from(filename: String) -> Result<Authenticator, Error> {
        let mut hash: HashMap<String, String> = HashMap::new();
        let file = File::open(filename)?;
        let reader = BufReader::new(file);


        for line in reader.lines(){
            let line = line.unwrap();
            let vec: Vec<&str> = line.split(";").collect();
            if vec.len() != 2 {
                return Err(Error::new(ErrorKind::Other, "Incorrect format"));
            }

            println!("Username: {}, Password: {}", vec.get(0).unwrap(), vec.get(1).unwrap());
            if hash.contains_key(&vec.get(0).unwrap().to_string()){
                return Err(Error::new(ErrorKind::Other, "Username already in use"));
            }
            hash.insert(vec.get(0).unwrap().to_string(), vec.get(1).unwrap().to_string());
        }

        Ok(Authenticator { accounts: hash })
    }

    pub fn account_is_valid(&self, username: &String, password: &String) -> bool {
        match self.accounts.get(username) {
            None => false,
            Some(pass) => pass == password,
        }
    }
}