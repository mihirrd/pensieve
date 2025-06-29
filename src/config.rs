use std::env;
// Logging
use tracing::{event, Level};

#[derive(Clone)]
pub enum Roles {
    Leader,
    Follower
}

pub struct Node {
    pub id : usize,
    pub role: Roles,
    pub port: String,
}

impl Node {
    fn new(id: usize, role: Roles, port:String) -> Node {
        Node {id, role, port}
    }
}

pub fn initialise_node() -> Node {
    let node_id: usize = env::var("NODE_ID")
        .expect("NODE_ID not set")
        .parse()
        .expect("NODE_ID must be a valid number");

     let port: String = env::var("PORT")
        .expect("Missing PORT")
        .to_string();

    event!(Level::INFO, "Node initialised - id : {:?}, port - {:?}", &node_id, port.clone());
    Node::new(node_id, Roles::Follower, port)
}
