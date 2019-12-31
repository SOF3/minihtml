use minihtml::html_string;

#[test]
fn test_basic() {
    let variable = "quz qux";
    let is_enabled = false;
    let ret: String = html_string! {
        html {
            head {
                title { +"Test title" };
            }
            body {
                img(src = "https://example.com");
                div(class = "foo bar") { +variable };
                button(disabled = !is_enabled) { +"the button" }
            }
        }
    };

    #[cfg_attr(rustfmt, rustfmt_skip)]
    assert_eq!(ret.as_str(), "<html>\
        <head>\
            <title>Test title</title>\
        </head>\
        <body>\
            <img src=\"https://example.com\"/>\
            <div class=\"foo bar\">quz qux</div>\
            <button disabled>the button</button>\
        </body>\
    </html>");
}
