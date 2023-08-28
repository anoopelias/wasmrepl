use wast::core::Expression;
use wast::core::Func;
use wast::core::Local;
use wast::kw;
use wast::parser::Parse;
use wast::parser::Parser;
use wast::parser::Result;

/// Parser for `local` instruction.
///
/// A single `local` instruction can generate multiple locals, hence this parser
pub struct LocalParser<'a> {
    /// All the locals associated with this `local` instruction.
    pub locals: Vec<Local<'a>>,
}

impl<'a> Parse<'a> for LocalParser<'a> {
    fn parse(parser: Parser<'a>) -> Result<Self> {
        let mut locals = Vec::new();
        parser.parse::<kw::local>()?;
        if !parser.is_empty() {
            let id: Option<_> = parser.parse()?;
            let name: Option<_> = parser.parse()?;
            let ty = parser.parse()?;
            let parse_more = id.is_none() && name.is_none();
            locals.push(Local { id, name, ty });
            while parse_more && !parser.is_empty() {
                locals.push(Local {
                    id: None,
                    name: None,
                    ty: parser.parse()?,
                });
            }
        }
        Ok(LocalParser { locals })
    }
}

pub enum Line<'a> {
    Expression(LineExpression<'a>),
    Func(Func<'a>),
}

pub struct LineExpression<'a> {
    pub locals: Vec<Local<'a>>,
    pub expr: Expression<'a>,
}

impl<'a> Parse<'a> for Line<'a> {
    fn parse(parser: Parser<'a>) -> Result<Self> {
        if parser.peek2::<kw::func>()? {
            let func = parser.parens(|p| p.parse::<Func>())?;
            return Ok(Line::Func(func));
        }

        // We need to parse locals explicitly because of this issue:
        // https://github.com/bytecodealliance/wasm-tools/issues/1156
        let mut locals = Vec::new();
        while parser.peek2::<kw::local>()? {
            parser.parens(|p| {
                locals.extend(p.parse::<LocalParser>()?.locals);
                Ok(())
            })?;
        }

        Ok(Line::Expression(LineExpression {
            locals,
            expr: parser.parse()?,
        }))
    }
}
