use std::collections::HashMap;
use serde::Serialize;
use crate::stream::Stream;

pub struct Ca65HtmlParser {
    input: Stream,
    start: usize,
    element_stack: Vec<String>,
    curr_key: String,
    curr_description: String,
    curr_href: String,
    is_href_code: bool,
}

#[derive(Serialize)]
pub struct KeywordInfo {
    documentation: String,
    snippet_type: String,
}

impl Ca65HtmlParser {
    pub fn new(input: Stream) -> Self {
        Self {
            input,
            start: 0,
            element_stack: Vec::new(),
            curr_key: String::from(""),
            curr_description: String::from(""),
            curr_href: String::from(""),
            is_href_code: false,
        }
    }
    pub fn parse_to_hashmap(&mut self) -> HashMap<String, KeywordInfo> {
        let mut hm = HashMap::<String, KeywordInfo>::new();
        while !self.input.at_end() {
            if let Some(c) = self.input.advance() {
                if c != '<' {
                    if !self.curr_key.is_empty() {
                        if self.is_top_element("code") {
                            if c != '\n' {
                                self.curr_description.push(c);
                            }
                        } else if !self.is_in_element_stack("h2") {
                            if self.is_top_element("p")
                                || self.is_top_element("a")
                                || self.is_top_element("ul")
                                || self.is_top_element("em")
                            {
                                if c == '&' { // every instance of & in ca65.html is an html escape
                                    self.add_html_escape_to_description();
                                } else {
                                    self.curr_description.push(c);
                                }
                            } else if self.is_top_element("pre") {
                                // ca65.html uses 8-space tabs for its code blocks
                                //  we only have small popup windows, so let's just bring the whole block closer to the left
                                if c == '\n' {
                                    self.curr_description.push('\n');
                                    const NUM_BEGINNING_TAB_SPACES: u8 = 2;
                                    for i in 0..8u8 {
                                        if self.input.match_char(' ') {
                                            if i < NUM_BEGINNING_TAB_SPACES {
                                                self.curr_description.push(' ');
                                            }
                                        } else {
                                            break;
                                        }
                                    }
                                } else {
                                    if c == '&' { // every instance of & in ca65.html is an html escape
                                        self.add_html_escape_to_description();
                                    } else {
                                        self.curr_description.push(c);
                                    }
                                }
                            }
                        }
                    }
                    continue
                }
                let mut is_closing_el = false;
                if self.input.peek() == Some('/') {
                    is_closing_el = true;
                    self.input.advance().expect("Unable to advance the stream in parse_to_hashmap");
                }
                let start = self.input.pos();
                self.consume_until_before(&[' ', '>']);
                let element_name = self.string_from(start).to_lowercase();
                if is_closing_el && element_name != "li" { // ca65.html doesn't always close it's <li>'s
                    if let Some(el) = self.element_stack.pop() {
                        if el == element_name && !self.curr_key.is_empty() {
                            match el.as_str() {
                                "h2" => self.curr_description.push_str("\n---\n"),
                                "blockquote" => self.curr_description.push_str("```\n"),
                                "a" => if !self.is_in_element_stack("h2") {
                                    if self.is_href_code {
                                        self.curr_description.push('`');
                                    }
                                    if !self.curr_href.is_empty() {
                                        self.curr_description.push_str(&format!("]({})", self.curr_href));
                                        self.curr_href.clear();
                                    }
                                },
                                "p" => self.curr_description.push_str("\n\n"),
                                "ul" => self.curr_description.push('\n'),
                                "code" => if !self.is_in_element_stack("blockquote") {
                                    if self.is_href_code {
                                        self.is_href_code = false;
                                    } else {
                                        self.curr_description.push('`');
                                    }
                                },
                                "em" => self.curr_description.push('_'),
                                _ => (),
                            }
                        }
                    }
                } else {
                    // make sure is not a self-closing html tag
                    if !Vec::<String>::from([
                        "!doctype".to_string(),
                        "meta".to_string(),
                        "link".to_string(),
                        "rel".to_string(),
                        "br".to_string(),
                        "hr".to_string(),
                        "li".to_string(), // ca65.html doesn't always close it's <li>'s
                    ]).contains(&element_name.to_lowercase()) {
                        self.element_stack.push(element_name.to_lowercase().clone());
                    }
                }
                if self.input.advance() == Some(' ') {
                    if element_name == "a" {
                        self.start = self.input.pos();
                        self.consume_until_after(&['"']);
                        if self.element_stack.len() > 1 && self.element_stack[self.element_stack.len() - 2] == "h2" {
                            if self.input.match_char('.') && self.current_string() == "NAME=\"." {
                                self.start = self.input.pos();
                                self.consume_until_before(&['"']);
                                self.curr_key = self.current_string().to_lowercase().clone();
                            }
                        } else if !self.curr_key.is_empty()
                            && !self.is_in_element_stack("h2")
                            && self.current_string() == "HREF=\""
                        {
                            self.start = self.input.pos();
                            self.consume_until_before(&['"']);
                            self.curr_href = self.current_string().clone();
                            if self.curr_href.starts_with('#') {
                                self.curr_href.insert_str(0, "https://cc65.github.io/doc/ca65.html");
                            } else if self.curr_href.starts_with("ca65.html") {
                                self.curr_href.insert_str(0, "https://cc65.github.io/doc/");
                            }
                            if self.curr_description.ends_with('`') {
                                self.is_href_code = true;
                                self.curr_description.pop();
                                self.curr_description.push_str("[`");
                            } else {
                                self.curr_description.push('[');
                            }
                        }
                    }
                    self.consume_until_after(&['>']);
                }
                if !is_closing_el && !self.curr_key.is_empty() {
                    match element_name.as_str() {
                        "h2" => {
                            hm.insert(self.curr_key.clone(), KeywordInfo {
                                documentation: self.curr_description.clone(),
                                snippet_type: String::new(),
                            });
                            self.curr_key = "".to_string();
                            self.curr_description = "".to_string();
                        },
                        "blockquote" => self.curr_description.push_str("```ca65"),
                        "code" => if !self.is_in_element_stack("blockquote") {
                            self.curr_description.push('`');
                        }
                        "li" => self.curr_description.push_str("\n- "),
                        "dd" => self.curr_description.push_str("\n\n"),
                        "dt" => self.curr_description.push_str("\n### "),
                        "em" => self.curr_description.push('_'),
                        _ => (),
                    }
                }
            }
        }
        hm
    }

    #[inline]
    fn add_html_escape_to_description(&mut self) {
        self.input.match_char('&');
        self.start = self.input.pos();
        self.consume_until_before(&[';']);
        match self.current_string().as_str() {
            "gt" => self.curr_description.push('>'),
            "lt" => self.curr_description.push('<'),
            "nbsp" => self.curr_description.push(' '),
            _ => (),
        }
        self.input.match_char(';');
    }
    #[inline]
    fn is_top_element(&self, el: &str) -> bool {
        self.element_stack.last() == Some(&el.to_string())
    }
    #[inline]
    fn is_in_element_stack(&self, el: &str) -> bool {
        self.element_stack.iter().any(|stack_el| stack_el == el)
    }
    #[inline]
    fn current_string(&self) -> String {
        String::from_utf8(self.input[self.start..self.input.pos()].to_vec()).expect("Failed to get string slice from stream")
    }
    #[inline]
    fn string_from(&self, start: usize) -> String {
        String::from_utf8(self.input[start..self.input.pos()].to_vec()).expect("Failed to get string slice from stream")
    }
    #[inline]
    fn consume_until_before(&mut self, terminators: &[char]) {
        loop {
            let c = self.input.peek().expect("Unable to peek next character of stream in consume_until_before");
            if terminators.to_vec().contains(&c) { break }
            self.input.advance().expect("Unable to advance the stream in consume_until_before");
        }
    }
    #[inline]
    fn consume_until_after(&mut self, terminators: &[char]) {
        while self.input.advance().is_some_and(|c| !terminators.to_vec().contains(&c)) {}
    }
}