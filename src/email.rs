use mailparse::*;
fn display_email(parsed: ParsedMail) {
    println!("mimetype: {}", parsed.ctype.mimetype);
    if parsed.ctype.mimetype == "text/plain" {
        let body = parsed.get_body().unwrap();
        let body = htmlescape::decode_html(&body).unwrap_or(body);
        println!("{}", body);
    } else if parsed.ctype.mimetype == "text/html" {
        let body = parsed.get_body().unwrap();
        let dom = tl::parse(&body, tl::ParserOptions::default()).unwrap();
        let mut last_node: &tl::Node = &tl::Node::Comment(tl::Bytes::new());
        for node in dom.nodes().iter() {
            //dbg!(node);
            match node {
                tl::Node::Tag(_) => last_node = node,
                tl::Node::Raw(b) => {
                    //dbg!(last_node);
                    let name = if let tl::Node::Tag(tag) = last_node {
                        tag.name().try_as_utf8_str().unwrap()
                    } else {
                        ""
                    };
                    if name != "style" && name != "title" && name != "" {
                        if b.try_as_utf8_str().unwrap().trim().len() > 0 {
                            //dbg!(b);
                            print!("{} ", b.try_as_utf8_str().unwrap());
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn display(buf: &[u8]) {
    let parsed = parse_mail(&buf).unwrap();
    let subject = parsed.headers.get_first_value("Subject");
    if let Some(s) = subject {
        println!("Subject: {}", s);
    }
    //display_email(parsed);
    //println!("Subparts: {}", parsed.subparts.len());
    //println!("mimetype: {}", parsed.ctype.mimetype);
    for part in parsed.subparts {
        display_email(part);
    }
}
