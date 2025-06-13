use bpaf::{params::ParseArgument, *};

pub fn index_name() -> ParseArgument<String> {
    long("index")
        .short('i')
        .argument::<String>("INDEX_NAME")
        .help("Name of the index to operate on")
}
