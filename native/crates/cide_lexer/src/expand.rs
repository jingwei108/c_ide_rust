//! 宏展开（对象式宏与参数化宏）。

use std::collections::HashSet;

use super::token::{Token, TokenType};
use super::Lexer;

impl Lexer {
    pub(crate) fn expand_macros(&self, tokens: Vec<Token>) -> Vec<Token> {
        self.expand_macros_inner(&tokens, &mut HashSet::new())
    }

    fn expand_macros_inner(&self, tokens: &[Token], expanding: &mut HashSet<String>) -> Vec<Token> {
        let mut result = Vec::new();
        let mut i = 0;
        while i < tokens.len() {
            let tok = &tokens[i];
            if tok.ty == TokenType::Identifier {
                if let Some(mdef) = self.macros.get(&tok.text) {
                    if expanding.contains(&tok.text) {
                        result.push(tok.clone());
                        i += 1;
                        continue;
                    }
                    if mdef.params.is_empty() {
                        // 对象式宏
                        expanding.insert(tok.text.clone());
                        let expanded = self.expand_macros_inner(&mdef.body, expanding);
                        expanding.remove(&tok.text);
                        for mut mt in expanded {
                            mt.line = tok.line;
                            mt.column = tok.column;
                            result.push(mt);
                        }
                        i += 1;
                        continue;
                    } else {
                        // 参数化宏：检查下一个 token 是否是 (
                        if i + 1 < tokens.len() && tokens[i + 1].ty == TokenType::LParen {
                            let mut args: Vec<Vec<Token>> = Vec::new();
                            let mut current_arg: Vec<Token> = Vec::new();
                            let mut depth = 1;
                            let mut j = i + 2; // skip LParen
                            while j < tokens.len() && depth > 0 {
                                match tokens[j].ty {
                                    TokenType::LParen => {
                                        depth += 1;
                                        current_arg.push(tokens[j].clone());
                                    }
                                    TokenType::RParen => {
                                        depth -= 1;
                                        if depth == 0 {
                                            break;
                                        }
                                        current_arg.push(tokens[j].clone());
                                    }
                                    TokenType::Comma if depth == 1 => {
                                        args.push(current_arg);
                                        current_arg = Vec::new();
                                    }
                                    _ => current_arg.push(tokens[j].clone()),
                                }
                                j += 1;
                            }
                            args.push(current_arg);

                            if args.len() != mdef.params.len() {
                                // 参数数量不匹配，不展开，保留原 token
                                result.push(tok.clone());
                                i += 1;
                                continue;
                            }

                            // 替换 body 中的参数
                            expanding.insert(tok.text.clone());
                            let mut substituted = Vec::new();
                            for bt in &mdef.body {
                                if bt.ty == TokenType::Identifier {
                                    if let Some(param_idx) = mdef.params.iter().position(|p| p == &bt.text) {
                                        substituted.extend(args[param_idx].iter().cloned());
                                        continue;
                                    }
                                }
                                substituted.push(bt.clone());
                            }

                            // H01: 参数化宏体为大括号块且调用后紧跟分号时，
                            // 动态包装为 do { ... } while(0)，使宏调用在 if/else 等语句中可正确解析。
                            let body_is_brace_block =
                                mdef.body.first().map(|t| t.ty == TokenType::LBrace).unwrap_or(false)
                                    && mdef.body.last().map(|t| t.ty == TokenType::RBrace).unwrap_or(false);
                            let followed_by_semicolon =
                                j + 1 < tokens.len() && tokens[j + 1].ty == TokenType::Semicolon;
                            if body_is_brace_block && followed_by_semicolon {
                                substituted.insert(
                                    0,
                                    Token {
                                        ty: TokenType::Do,
                                        text: "do".to_string(),
                                        line: tok.line,
                                        column: tok.column,
                                    },
                                );
                                substituted.push(Token {
                                    ty: TokenType::While,
                                    text: "while".to_string(),
                                    line: tok.line,
                                    column: tok.column,
                                });
                                substituted.push(Token {
                                    ty: TokenType::LParen,
                                    text: "(".to_string(),
                                    line: tok.line,
                                    column: tok.column,
                                });
                                substituted.push(Token {
                                    ty: TokenType::Number,
                                    text: "0".to_string(),
                                    line: tok.line,
                                    column: tok.column,
                                });
                                substituted.push(Token {
                                    ty: TokenType::RParen,
                                    text: ")".to_string(),
                                    line: tok.line,
                                    column: tok.column,
                                });
                            }

                            let expanded = self.expand_macros_inner(&substituted, expanding);
                            expanding.remove(&tok.text);
                            for mut mt in expanded {
                                mt.line = tok.line;
                                mt.column = tok.column;
                                result.push(mt);
                            }
                            i = j + 1;
                            continue;
                        }
                    }
                }
            }
            result.push(tok.clone());
            i += 1;
        }
        result
    }
}
