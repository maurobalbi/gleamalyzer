use crate::SyntaxKind::{self, *};
use crate::{GleamLanguage, SyntaxNode, SyntaxToken};
use rowan::ast::support::{child, children};
use rowan::NodeOrToken;

pub use rowan::ast::{AstChildren, AstNode};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BinaryOpKind {
    Imply,
    Or,
    And,

    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,

    Update,
    Concat,

    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnaryOpKind {
    Not,
    Negate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiteralKind {
    Int,
    Float,
    String,
}

trait NodeWrapper {
    const KIND: SyntaxKind;
}

macro_rules! enums {
    ($($name:ident { $($variant:ident,)* },)*) => {
        $(
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub enum $name {
            $($variant($variant),)*
        }

        impl AstNode for $name {
            type Language = GleamLanguage;

            fn can_cast(kind: SyntaxKind) -> bool
                where Self: Sized
            {
                matches!(kind, $(<$variant as NodeWrapper>::KIND)|*)
            }

            fn cast(node: SyntaxNode) -> Option<Self>
            where
                Self: Sized
            {
                match node.kind() {
                    $(<$variant as NodeWrapper>::KIND => Some(Self::$variant($variant(node))),)*
                    _ => None,
                }
            }

            fn syntax(&self) -> &SyntaxNode {
                match self {
                    $(Self::$variant(e) => &e.0,)*
                }
            }
        }
        )*
    };
}

macro_rules! asts {
    (
        $(
            $kind:ident = $name:ident $([$trait:tt])?
            { $($impl:tt)* },
        )*
    ) => {
        $(
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub struct $name(SyntaxNode);

        impl $name {
            ast_impl!($($impl)*);
        }

        $(impl $trait for $name {})*

        impl NodeWrapper for $name {
            const KIND: SyntaxKind = SyntaxKind::$kind;
        }

        impl AstNode for $name {
            type Language = GleamLanguage;

            fn can_cast(kind: SyntaxKind) -> bool
                where Self: Sized
            {
                kind == SyntaxKind::$kind
            }

            fn cast(node: SyntaxNode) -> Option<Self>
            where
                Self: Sized
            {
                if node.kind() == SyntaxKind::$kind {
                    Some(Self(node))
                } else {
                    None
                }
            }

            fn syntax(&self) -> &SyntaxNode {
                &self.0
            }
        }
        )*
    };
}

macro_rules! ast_impl {
    () => {};
    ($field:ident: $ast:ident, $($tt:tt)*) => {
        pub fn $field(&self) -> Option<$ast> { child(&self.0) }
        ast_impl!($($tt)*);
    };
    ($field:ident[$k:tt]: $ast:ident, $($tt:tt)*) => {
        pub fn $field(&self) -> Option<$ast> { children(&self.0).nth($k) }
        ast_impl!($($tt)*);
    };
    ($field:ident: [$ast:ident], $($tt:tt)*) => {
        pub fn $field(&self) -> AstChildren<$ast> { children(&self.0) }
        ast_impl!($($tt)*);
    };
    ($field:ident: T![$tok:tt], $($tt:tt)*) => {
        pub fn $field(&self) -> Option<SyntaxToken> {
            token(&self.0, T![$tok])
        }
        ast_impl!($($tt)*);
    };
    ($field:ident[$k:tt]: T![$tok:tt], $($tt:tt)*) => {
        pub fn $field(&self) -> Option<SyntaxToken> {
            self.0
                .children_with_tokens()
                .filter_map(|it| it.into_token())
                .filter(|it| it.kind() == T![$tok])
                .nth($k)
        }
        ast_impl!($($tt)*);
    };
    ($($item:item)*) => {
        $($item)*
    };
}

enums! {
    Statement {
        ModuleConstant,
        Import,
    },
    ConstantValue {
        Literal,
        Tuple,
        List,
    },
    TypeAnnotation {
        FnType,
        VarType,
        TupleType,
        ConstructorType,
    },
}

asts! {
    LIST = List {
        elements: [ConstantValue],
    },
    LITERAL = Literal {
        pub fn token(&self) -> Option<SyntaxToken> {
            self.0.children_with_tokens().find_map(NodeOrToken::into_token)
        }

        pub fn kind(&self) -> Option<LiteralKind> {
            Some(match self.token()?.kind() {
                INTEGER => LiteralKind::Int,
                FLOAT => LiteralKind::Float,
                STRING => LiteralKind::String,
                _ => return None,
            })
        }
    },
    IMPORT = Import {
        module: ImportModule,
    },
    IMPORT_MODULE = ImportModule {
        module_path: [Path],
        as_name: Name,
        unqualified: [UnqualifiedImport],
    },
    SOURCE_FILE = SourceFile {
        statements: [TargetGroup],
    },
    MODULE_NAME = ModuleName {
        pub fn token(&self) -> Option<SyntaxToken> {
            self.0.children_with_tokens().find_map(NodeOrToken::into_token)
        }
    },
    MODULE_CONSTANT = ModuleConstant {
        name: Name,
        value: ConstantValue,
        annotation: TypeAnnotation,
        pub fn is_public(&self) -> bool {
            self.syntax().children_with_tokens().find(|it| it.kind() == T!["pub"]).is_some()
        }
    },
    NAME = Name {
        pub fn token(&self) -> Option<SyntaxToken> {
            self.0.children_with_tokens().find_map(NodeOrToken::into_token)
        }
    },
    PATH = Path {
        pub fn token(&self) -> Option<SyntaxToken> {
            self.0.children_with_tokens().find_map(NodeOrToken::into_token)
        }
    },
    UNQUALIFIED_IMPORT = UnqualifiedImport {
      name: Name,
      as_name[1]: Name,
    },
    PARAM = Param {
        // pat: Pat,
        ty: TypeAnnotation,
    },
    PARAM_LIST = ParamList {
        params: [Param],
    },
    TARGET = Target {
        name: Name,
    },
    TARGET_GROUP = TargetGroup {
        target: Target,
        statements: [Statement],
    },
    TUPLE = Tuple {
        elements: [ConstantValue],
    },
    CONSTRUCTOR_TYPE = ConstructorType {
      constructor: Name,
      module: ModuleName,
    },
    TUPLE_TYPE = TupleType{
      field_types: [TypeAnnotation],
    },
    FN_TYPE = FnType {
        param_list: ParamList,
        return_: TypeAnnotation,
    },
    VAR_TYPE = VarType {
        name: Name,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::parse;

    trait HasSyntaxNode {
        fn has_syntax_node(&self) -> &SyntaxNode;
    }

    trait AstTest {
        fn should_eq(&self, expect: &str);
    }

    impl AstTest for SyntaxNode {
        #[track_caller]
        fn should_eq(&self, expect: &str) {
            assert_eq!(self.to_string().trim(), expect);
        }
    }

    impl AstTest for SyntaxToken {
        #[track_caller]
        fn should_eq(&self, expect: &str) {
            assert_eq!(self.to_string(), expect);
        }
    }

    #[test]
    fn apply() {
        let e = parse::<ImportModule>("import aa/a.{m as a}");
        println!("{:?}", e.unqualified().next().unwrap().as_name());
        // println!("{:?}", e.statements().next().unwrap().syntax());
    }

    #[test]
    fn assert() {
        let e = crate::parse_file("import aa/a.{m as a, M as A as e");
        for error in e.errors() {
            println!("{}", error);
        }
        println!("{:#?}", e.syntax_node())
    }

    #[test]
    fn pub_const() {
        let e = parse::<ModuleConstant>("pub const a = \"123\"");
        e.name().unwrap().syntax().should_eq("a");
        assert!(e.is_public());
    }

    #[test]
    fn const_tuple() {
        let e = parse::<Tuple>("const a = #(#(2,3),2)");
        let mut iter = e.elements();
        iter.next().unwrap().syntax().should_eq("#(2,3)");
        iter.next().unwrap().syntax().should_eq("2");
        assert!(iter.next().is_none())
    }

    #[test]
    fn module() {
        let e =
            parse::<SourceFile>("if erlang {const a = 1} const b = 2 if javascript {const c = 3}");
        let mut iter = e.statements();
        iter.next()
            .unwrap()
            .syntax()
            .should_eq("if erlang {const a = 1}");
        assert!(iter.next().is_some());
        assert!(iter.next().is_some());
        assert!(iter.next().is_none());
    }

    #[test]
    fn target_group() {
        let e =
            parse::<TargetGroup>("if erlang {const a = 1} const b = 2 if javascript {const c = 3}");
        e.target().unwrap().syntax().should_eq("erlang");
        let mut iter = e.statements();
        iter.next().unwrap().syntax().should_eq("const a = 1");
    }

    #[test]
    fn fn_type_ann() {
        let e = parse::<FnType>("const a: fn(Int, String) -> Cat = 1");
        e.return_().unwrap().syntax().should_eq("Cat");
        let mut iter = e.param_list().unwrap().params();
        iter.next().unwrap().syntax().should_eq("Int");
        iter.next().unwrap().syntax().should_eq("String");
    }

    #[test]
    fn tuple_type_ann() {
        let e = parse::<TupleType>("const a: #(Int, String) = 1");
        let mut iter = e.field_types();
        iter.next().unwrap().syntax().should_eq("Int");
        iter.next().unwrap().syntax().should_eq("String");
    }

    #[test]
    fn constructor_module_type() {
        let e = parse::<ModuleConstant>("const a: gleam.Int = 1");
        e.annotation().unwrap().syntax().should_eq("gleam.Int")
    }

    #[test]
    fn module_constructor_type() {
        let e = parse::<ConstructorType>("const a : gleam.Int = 1");
        e.constructor().unwrap().syntax().should_eq("Int");
        e.module().unwrap().syntax().should_eq("gleam");
    }

    #[test]
    fn import_module() {
        let e = parse::<ImportModule>("import aa/a");
        let mut iter = e.module_path();
        iter.next().unwrap().syntax().should_eq("aa");
        iter.next().unwrap().syntax().should_eq("a");
        assert!(iter.next().is_none());
    }

    #[test]
    fn import_unqualified() {
        let e = parse::<ImportModule>("import aa/a.{m as a, M as A}");
        let mut iter = e.unqualified();
        let fst = iter.next().unwrap();
        let snd = iter.next().unwrap();

        fst.as_name().unwrap().syntax().should_eq("a");
        fst.name().unwrap().syntax().should_eq("m");
        snd.as_name().unwrap().syntax().should_eq("A");
        snd.name().unwrap().syntax().should_eq("M");
        assert!(iter.next().is_none());
    }

    #[test]
    fn import_qualified_as() {
        let e = parse::<ImportModule>("import aa/a.{m as a, M as A} as e");

        e.as_name().unwrap().syntax().should_eq("e");
    }
    //     let mut iter = e.bindings();
    //     iter.next().unwrap().syntax().should_eq("a = let { };");
    //     iter.next().unwrap().syntax().should_eq("b = rec { };");
    // }
}
