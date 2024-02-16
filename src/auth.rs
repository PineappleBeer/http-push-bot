pub fn auth(key:String) -> bool {
    let res:bool = key == String::from("halo-world");

    res
}