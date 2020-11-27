use super::*;

#[derive(Serialize, Debug, Clone)]
pub struct Operation {
    left: Option<Box<Operand>>,
    right: Option<Box<Operand>>,
    op: Operator,
}

impl Operation {
    fn find_split_index(tokens: &[RaToken]) -> Option<usize> {
        let mut left_end_index = tokens.len();
        let mut result = Group::accepts_tokens(tokens);

        while !result {
            left_end_index = {
                if left_end_index == tokens.len() {
                    tokens.len() - 1 - 2
                }
                else {
                    left_end_index
                }
            };

            result = {
                Operand::accepts_tokens(&tokens[..left_end_index])
                    && ({
                        Operator::accepts_tokens(&tokens[left_end_index..left_end_index + 1])
                            && Operand::accepts_tokens(&tokens[left_end_index..left_end_index + 2])
                    } || {
                        Operator::accepts_tokens(&tokens[left_end_index..left_end_index + 2])
                            && tokens.len() >= left_end_index + 4
                            && Operand::accepts_tokens(&tokens[left_end_index..left_end_index + 3])
                    })
            };
            if left_end_index == 0 {
                break;
            } else {
                left_end_index -= 1;
            }
        }

        if result {
            Some(left_end_index)
        } else {
            None
        }
    }
}

impl Expression for Operation {
    fn accepts_tokens(tokens: &[RaToken]) -> bool {
        match tokens {
            t if t.len() >= 3 && Operator::accepts_tokens(&t[..=1]) => {
                Operand::accepts_tokens(&t[2..])
            }
            t if t.len() >= 2 && Operator::accepts_tokens(&t[..=0]) => {
                Operand::accepts_tokens(&t[1..])
            }
            t if t.len() >= 3 && Operand::accepts_tokens(&t[..=0]) => {
                (Operator::accepts_tokens(&t[1..2])
                    && Operand::accepts_tokens(&t[2..])
                    && t.len() == 3)
                    || (Operator::accepts_tokens(&t[1..=2])
                        && Operand::accepts_tokens(&t[3..])
                        && t.len() == 4)
            }
            t if t.len() >= 3 => Self::find_split_index(tokens).is_some(),
            _ => false,
        }
    }

    fn parse(tokens: &[RaToken]) -> Result<Self, Vec<ParserError>> {
        match tokens {
            t if t.len() >= 3 && Operator::accepts_tokens(&t[..=1]) => {
                let op = Operator::parse(&t[..=1])?;
                let right = Some(Box::new(Operand::parse(&t[2..])?));
                Ok(Self {
                    op,
                    right,
                    left: None,
                })
            }
            t if t.len() >= 2 && Operator::accepts_tokens(&t[..=0]) => {
                let op = Operator::parse(&t[..=0])?;
                let right = Some(Box::new(Operand::parse(&t[1..])?));
                Ok(Self {
                    op,
                    right,
                    left: None,
                })
            }
            t if t.len() >= 3 && Operand::accepts_tokens(&t[..=0]) => {
                let left = Some(Box::new(Operand::parse(&t[..=0])?));
                let mut op_end_index = 0;
                let op = match Operator::parse(&t[1..3]) {
                    Ok(op) => {
                        op_end_index = 3;
                        Ok(op)
                    }
                    Err(mut e) => match Operator::parse(&t[1..2]) {
                        Ok(op) => {
                            op_end_index = 2;
                            Ok(op)
                        }
                        Err(e2) => {
                            e.extend(e2);
                            Err(e)
                        }
                    },
                }?;
                let right = Some(Box::new(Operand::parse(&t[op_end_index..])?));

                Ok(Self { left, op, right })
            }
            t if t.len() >= 3 => match Self::find_split_index(t) {
                Some(left_end_index) => {

                    if left_end_index == t.len() {
                        let left = Some(Box::new(Operand::parse(t)?));

                        Ok(Self {
                            left,
                            right: None,
                            op: Operator::None
                        })
                    }
                    else {
                        todo!("parse Operation");
                    }
                    // if Operator::accepts_tokens(&tokens[left_end_index..left_end_index + 1])
                    //     && Operand::accepts_tokens(&tokens[left_end_index..left_end_index + 2])
                    // {
                    //     let op = Operator::parse(&tokens[left_end_index..left_end_index + 1])?;
                    //     let right = Some(Box::new(Operand::parse(&tokens[left_end_index..left_end_index + 2])?));
                    //     Ok(Self {
                    //         op, right,
                    //         left: None
                    //     })
                    // } else if Operator::accepts_tokens(&tokens[left_end_index..left_end_index + 2])
                    //     && tokens.len() >= left_end_index + 4
                    //     && Operand::accepts_tokens(&tokens[left_end_index..left_end_index + 3])
                    // {

                    // } else {
                    //     Err(vec![ParserError::InvalidExpression(
                    //         t[0].position.0,
                    //         Backtrace::new(),
                    //     )])
                    // }
                }
                None => Err(vec![ParserError::InvalidExpression(
                    t[0].position.0,
                    Backtrace::new(),
                )]),
            },
            _ => Err(vec![ParserError::ExpectedAGotB(
                format!("{:?}", Vec::<()>::new()),
                format!("{:?}", tokens),
                tokens.first().unwrap_or(&RaToken::default()).position.0,
                Backtrace::new(),
            )]),
        }
    }
    fn level(&self) -> u16 {
        self.op.level()
    }

    fn position(&self) -> (Position, Position) {
        (
            match self.left.as_ref() {
                Some(l) => l.position().0,
                None => self.op.position().0,
            },
            match self.right.as_ref() {
                Some(r) => r.position().1,
                None => self.op.position().1,
            },
        )
    }
}