use wast::core::Expression;
use wast::core::Local;
use wast::kw;
use wast::parser::Parse;
use wast::parser::Parser;
use wast::parser::Result;

fn parse_local(parser: Parser) -> Result<Local> {
    parser.parse::<kw::local>()?;
    let id: Option<_> = parser.parse()?;
    let name: Option<_> = parser.parse()?;
    let ty = parser.parse()?;
    Ok(Local { id, name, ty })
}

fn parse_locals<'a>(parser: Parser<'a>) -> Result<Vec<Local<'a>>> {
    let mut locals = Vec::new();
    while parser.peek2::<kw::local>()? {
        parser.parens(|p| {
            locals.push(parse_local(p)?);
            Ok(())
        })?;
    }
    Ok(locals)
}

pub struct LineParser<'a> {
    pub locals: Vec<Local<'a>>,
    pub expr: Expression<'a>,
}

impl<'a> Parse<'a> for LineParser<'a> {
    fn parse(parser: Parser<'a>) -> Result<Self> {
        // We need to parse locals explicitly because of this issue:
        // https://github.com/bytecodealliance/wasm-tools/issues/1156
        let locals = parse_locals(parser)?;
        Ok(LineParser {
            locals,
            expr: parser.parse()?,
        })
    }
}
