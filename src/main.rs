#![warn(clippy::pedantic, clippy::nursery)]

use std::{fmt::Debug, fs, path::Path, vec};

// use walkdir::WalkDir;
use lib_ruby_parser::{
    nodes::{Alias, Casgn, Class, Const, Def, Defs, Module, Send, Sym},
    source::DecodedInput,
    traverse::visitor::{
        visit_alias, visit_casgn, visit_class, visit_def, visit_defs, visit_module, visit_send,
        Visitor,
    },
    Node, Parser, ParserOptions, ParserResult,
};

#[derive(Debug)]
enum Kind {
    Class,
    Module,
    Method,
    Constant,
    SingletonMethod,
    Alias,
    // Accessor,
}

#[derive(Debug)]
struct Definition {
    name: String,
    file: String,
    source: String,
    kind: Kind,
}

struct TagsCollector {
    input: DecodedInput,
    aliases: Vec<Definition>,
    classes: Vec<Definition>,
    constants: Vec<Definition>,
    methods: Vec<Definition>,
    modules: Vec<Definition>,
}

impl Debug for TagsCollector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tags")
            .field("aliases", &self.aliases)
            .field("classes", &self.classes)
            .field("constants", &self.constants)
            .field("methods", &self.methods)
            .field("modules", &self.modules)
            .finish()
    }
}

impl TagsCollector {
    fn fetch_const_name(name: &Node) -> String {
        match name {
            Node::Const(Const { name, .. }) => name.clone(),
            Node::Sym(Sym { name, .. }) => name.to_string_lossy(),
            other => panic!("Don't know how to fetch const name from {:?}", other),
        }
    }
}

impl Visitor for TagsCollector {
    fn on_alias(&mut self, node: &Alias) {
        self.aliases.push(Definition {
            name: Self::fetch_const_name(&node.to),
            file: self.input.name.clone(),
            source: {
                let loc = &node.expression_l;
                let source = loc.source(&self.input).unwrap();
                source.split('\n').next().unwrap_or("").to_string()
            },
            kind: Kind::Alias,
        });
        visit_alias(self, node);
    }

    fn on_casgn(&mut self, node: &Casgn) {
        self.constants.push(Definition {
            name: node.name.clone(),
            file: self.input.name.clone(),
            source: {
                let loc = &node.expression_l;
                let source = loc.source(&self.input).unwrap();
                source.split('\n').next().unwrap_or("").to_string()
            },
            kind: Kind::Constant,
        });
        visit_casgn(self, node);
    }

    fn on_class(&mut self, node: &Class) {
        self.classes.push(Definition {
            name: Self::fetch_const_name(&node.name),
            file: self.input.name.clone(),
            source: {
                let loc = &node.expression_l;
                let source = loc.source(&self.input).unwrap();
                source.split('\n').next().unwrap_or("").to_string()
            },
            kind: Kind::Class,
        });
        visit_class(self, node);
    }

    fn on_def(&mut self, node: &Def) {
        self.methods.push(Definition {
            name: node.name.clone(),
            file: self.input.name.clone(),
            source: {
                let loc = &node.expression_l;
                let source = loc.source(&self.input).unwrap();
                source.split('\n').next().unwrap_or("").to_string()
            },
            kind: Kind::Method,
        });
        visit_def(self, node);
    }

    fn on_defs(&mut self, node: &Defs) {
        self.methods.push(Definition {
            name: node.name.clone(),
            file: self.input.name.clone(),
            source: {
                let loc = &node.expression_l;
                let source = loc.source(&self.input).unwrap();
                source.split('\n').next().unwrap_or("").to_string()
            },
            kind: Kind::SingletonMethod,
        });
        visit_defs(self, node);
    }

    fn on_module(&mut self, node: &Module) {
        self.modules.push(Definition {
            name: Self::fetch_const_name(&node.name),
            file: self.input.name.clone(),
            source: {
                let loc = &node.expression_l;
                let source = loc.source(&self.input).unwrap();
                source.split('\n').next().unwrap_or("").to_string()
            },
            kind: Kind::Module,
        });
        visit_module(self, node);
    }

    fn on_send(&mut self, node: &Send) {
        if node.method_name.starts_with("attr_") {
            self.methods.extend(node.args.iter().filter_map(|arg| {
                if let Node::Sym(Sym { name, .. }) = arg {
                    Some(Definition {
                        name: name.to_string_lossy(),
                        file: self.input.name.clone(),
                        source: {
                            let expression_loc = &node.expression_l;
                            let source = expression_loc.source(&self.input).unwrap();
                            source.split('\n').next().unwrap_or("").to_string()
                        },
                        kind: Kind::Method,
                    })
                } else {
                    None
                }
            }))
        }
        visit_send(self, node);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // for entry in WalkDir::new("../aoc/2021/ruby/lib")
    //     .follow_links(true)
    //     .into_iter()
    //     .filter_map(|e| e.ok())
    // {
    //     let f_name = entry.path().to_string_lossy();

    //     if f_name.ends_with(".rb") {
    //         println!("{}", f_name);
    //     }
    // }
    let file = Path::new("../aoc/2021/ruby/lib/day10.rb");
    let file_contents = fs::read(&file)?;
    let options = ParserOptions {
        buffer_name: file.to_str().unwrap().to_string(),
        record_tokens: false,
        ..Default::default()
    };
    let parser = Parser::new(file_contents, options);
    let ParserResult { ast, input, .. } = parser.do_parse();
    let ast = ast.unwrap();

    let mut collector = TagsCollector {
        input,
        aliases: vec![],
        classes: vec![],
        constants: vec![],
        methods: vec![],
        modules: vec![],
    };
    collector.visit(&ast);

    println!("{:#?}", collector);

    Ok(())
}
