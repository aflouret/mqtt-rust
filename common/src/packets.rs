mod packet_flags;

struct Connect {
    client_id: String,
    username: String,
    password: String,
    connect_flags: ConnectFlags,
    last_will_message: String,
    last_will_topic: String,
    // keep alive?
}

impl Connect {
    fn new(client_id: String,
        username: String,
        password: String,
        connect_flags: ConnectFlags,
        last_will_message: String,
        last_will_topic: String) -> Connect {
            Connect {
                client_id, username, password, last_will_message, last_will_topic
            }
        }
}

struct Connack {
    flags: Flags,
}

struct Publish {
    flags: Flags,
}

struct Puback {
    flags: Flags,
}

struct Subscribe {
    flags: Flags,
}

struct Unsubscribe {
    flags: Flags,
}

struct Suback {
    flags: Flags,
}

struct Unsuback {
    flags: Flags,
}

struct Disconnect {
    flags: Flags,
}

pub enum Packet {
    Connect(Connect),
    Connack(Connack),
    Publish(Publish),
    Puback(Puback),
    Subscribe(Subscribe),
    Unsubscribe(Unsubscribe),
    Suback(Suback),
    Unsuback(Unsuback),
    Disconnect(Disconnect),
}

impl Packet {
    
}