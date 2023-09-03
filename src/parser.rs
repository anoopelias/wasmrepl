use wast::core::Expression;
use wast::core::Func;
use wast::core::Local;
use wast::core::LocalParser;
use wast::kw;
use wast::parser::Parse;
use wast::parser::Parser;
use wast::parser::Result;

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

#[cfg(test)]
mod tests {
    use wast::{
        core::Instruction,
        parser::{parse, ParseBuffer},
    };

    use crate::parser::Line;

    #[test]
    fn test_line_parse_expr() {
        let buf = ParseBuffer::new("(i32.const 32)").unwrap();
        let lp = parse::<Line>(&buf).unwrap();

        if let Line::Expression(line_expr) = lp {
            assert_eq!(line_expr.expr.instrs.len(), 1);
            if let Instruction::I32Const(i) = line_expr.expr.instrs[0] {
                assert_eq!(i, 32);
            } else {
                panic!("Expected Instruction::I32Const");
            }
        } else {
            panic!("Expected Line::Expression");
        }
    }

    #[test]
    fn test_line_parse_local() {
        let buf = ParseBuffer::new("(local $num i32)").unwrap();
        let lp = parse::<Line>(&buf).unwrap();

        if let Line::Expression(line_expr) = lp {
            assert_eq!(line_expr.locals.len(), 1);
            let lc = line_expr.locals.get(0).unwrap();
            assert_eq!(lc.id.unwrap().name(), "num");
        } else {
            panic!("Expected Line::Expression");
        }
    }

    #[test]
    fn test_line_parse_func() {
        let buf = ParseBuffer::new("(func $f (i32.const 44))").unwrap();
        let lp = parse::<Line>(&buf).unwrap();

        if let Line::Func(func) = lp {
            assert_eq!(func.id.unwrap().name(), "f");
        } else {
            panic!("Expected Line::Expression");
        }
    }
}
