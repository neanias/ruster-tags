use std::{
    fs,
    path::{self, PathBuf},
    vec,
};

// use walkdir::WalkDir;
use lib_ruby_parser::{
    nodes::{Class, Const, Def, Module},
    traverse::visitor::{visit_class, visit_def, visit_module, Visitor},
    Node, Parser, ParserOptions, ParserResult, source::DecodedInput,
};

#[allow(dead_code)]
#[derive(Debug)]
enum Kind {
    Class,
    Module,
    Method,
    Constant,
    SingletonMethod,
    Alias,
    Accessor,
}

#[allow(dead_code)]
#[derive(Debug)]
struct Definition {
    name: String,
    file: path::PathBuf,
    line_number: usize,
    source: String,
    kind: Kind,
}

#[derive(Debug)]
struct TagsCollector {
    input: DecodedInput,
    classes: Vec<Definition>,
    methods: Vec<Definition>,
    modules: Vec<Definition>,
}

impl TagsCollector {
    fn file_path() -> PathBuf {
        PathBuf::from("../freeagent_training/lib/freeagent_training/loader.rb")
    }

    fn fetch_const_name(name: &Node) -> String {
        match name {
            Node::Const(Const { name, .. }) => name.to_owned(),
            other => panic!("Don't know how to fetch const name from {:?}", other),
        }
    }
}

impl Visitor for TagsCollector {
    fn on_class(&mut self, node: &Class) {
        self.classes.push(Definition {
            name: TagsCollector::fetch_const_name(&node.name),
            file: PathBuf::from(&self.input.name),
            line_number: node.keyword_l.begin,
            source: node.expression_l.source(&self.input).unwrap(),
            kind: Kind::Method,
        });
        visit_class(self, node);
    }

    fn on_def(&mut self, node: &Def) {
        self.methods.push(Definition {
            name: node.name.to_owned(),
            file: TagsCollector::file_path(),
            line_number: node.keyword_l.begin,
            source: node.expression_l.source(&self.input).unwrap(),
            kind: Kind::Method,
        });
        visit_def(self, node);
    }

    fn on_module(&mut self, node: &Module) {
        self.modules.push(Definition { 
            name: TagsCollector::fetch_const_name(&node.name),
            file: TagsCollector::file_path(),
            line_number: node.keyword_l.begin,
            source: node.expression_l.source(&self.input).unwrap(),
            kind: Kind::Module,
        });
        visit_module(self, node);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // for entry in WalkDir::new("../freeagent_training")
    //     .follow_links(true)
    //     .into_iter()
    //     .filter_map(|e| e.ok())
    // {
    //     let f_name = entry.path().to_string_lossy();

    //     if f_name.ends_with(".rb") {
    //         println!("{}", f_name);
    //     }
    // }
    let options = ParserOptions {
        buffer_name: "(eval)".to_string(),
        record_tokens: false,
        ..Default::default()
    };
    let file_contents = fs::read("../freeagent_training/lib/freeagent_training/loader.rb")?;
    let parser = Parser::new(file_contents, options);
    let ParserResult { ast, input, .. } = parser.do_parse();
    let ast = ast.unwrap();

    let mut collector = TagsCollector {
        input,
        classes: vec![],
        methods: vec![],
        modules: vec![],
    };
    collector.visit(&ast);

    println!("{:#?}", collector);

    Ok(())
}
