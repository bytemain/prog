pub fn clone(
    url: &String,
    rest: &[String],
    target_path: &str,
) -> anyhow::Result<(), anyhow::Error> {
    let mut list = vec!["git", "clone", url, target_path];
    let mut rest_str: Vec<&str> = rest.iter().map(|x| x.as_str()).collect();
    list.append(&mut rest_str);

    crate::helpers::shell::run(&list.join(" "))
}
