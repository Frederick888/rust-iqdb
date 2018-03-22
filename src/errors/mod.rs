use reqwest;

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        Another(reqwest::Error);
        Io(::std::io::Error);
    }

    errors {
        WaitTooLong(t: u32) {
            description("queue is too long")
            display("queue is too long: '{}'", t)
        }
    }
}
