use std::{collections::HashMap, str::Chars};

use parser_combinator::{either::Either, Parse};

use crate::{
    definition::*,
    expression::{EExpression, ExpressionSyntaxTree},
    parser_common::*,
};

pub type DefinitionParseResult<'a> = Result<(Definition<String>, State, Chars<'a>), ParseErrors>;

type DefinitionItem = Either<
    (String, (Vec<String>, ExpressionSyntaxTree<String>)),
    (String, ExpressionSyntaxTree<String>),
>;

impl<'a> Parse<'a, Chars<'a>, State, Definition<String>, ParseErrors> for EDefinition {
    fn parse(&self, input: Chars<'a>, state: State) -> DefinitionParseResult<'a> {
        let const_def = LowerCaseName
            .triple(Equal, EExpression)
            .pair(SemiColon)
            .first()
            .transform(|(name, _, expr)| (name, expr));

        let fname = LowerCaseName
            .pair(
                LParen
                    .triple(LowerCaseName.separated_by(Comma), RParen)
                    .second()
                    .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end)),
            )
            .transform(|(name, args)| {
                let mut ret = args
                    .1
                    .into_iter()
                    .map(|(_, arg)| arg)
                    .collect::<Vec<String>>();
                ret.insert(0, args.0);
                (name, ret)
            })
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end));

        let fun_def = fname
            .triple(Equal, EExpression)
            .transform(|(name, _, expr)| (name, expr))
            .pair(SemiColon)
            .first()
            .transform(|((name, arg_vec), expr)| (name, (arg_vec, expr)))
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end));

        fun_def
            .either(const_def)
            .zero_or_more()
            .transform(
                |x: Vec<DefinitionItem>| {
                    let mut const_map = HashMap::new();
                    let mut fun_map = HashMap::new();

                    x.into_iter().for_each(|def| match def {
                        Either::Left((name, (args, expr))) => {
                            fun_map.insert((name, args.len()), (args, expr));
                        }
                        Either::Right((name, expr)) => {
                            const_map.insert(name, expr);
                        }
                    });
                    Definition::new(fun_map, const_map)
                },
            )
            .with_error_using_state(|_, s, _| ParseErrors::Generic(s.start, s.end))
            .parse(input, state)
    }
}
