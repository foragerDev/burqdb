

pub struct BTree {
    // pub root_node: Option<Box<BTreeNode>>,
}


impl BTree {
    pub fn new() -> Self {
        BTree {}
    }

    pub fn load(path: &str) -> Self {
        println!("Loading B-tree from path: {}", path);
        BTree {}
    }

    pub fn insert(&self, key: String, value: String) {
        // Insert key-value pair into the B-tree
        println!("Inserting key: {}, value: {}", key, value);
    }

    pub fn get(&self, key: &String) -> Option<String> {
        // Retrieve value by key from the B-tree
        println!("Getting value for key: {}", key);
        None
    }

    pub fn delete(&self, key: &String) {
        // Delete key-value pair from the B-tree
        println!("Deleting key: {}", key);
    }

    pub fn range_query(&self, start: &String, end: &String) -> Vec<(String, String)> {
        // Perform a range query from start to end keys
        println!("Performing range query from {} to {}", start, end);
        vec![]
    }


    pub fn range_query_filter<F>(&self, start: &String, end: &String, filter: F) -> Vec<(String, String)>
    where
        F: Fn(&String, &String) -> bool,
    {
        // Perform a range query with a filter function
        println!("Performing filtered range query from {} to {}", start, end);
        vec![]
    }

    

    

}