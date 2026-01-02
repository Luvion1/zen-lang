use crate::lexer::lexer::Lexer;
use crate::parser::parser::Parser;
use crate::ownership::OwnershipChecker;

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_code(code: &str) -> crate::ast::program::Program {
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        parser.parse().unwrap()
    }

    #[test]
    fn test_basic_ownership() {
        let code = r#"
            fn main() -> i32 {
                let x = 42
                let y = <-x
                return 0
            }
        "#;
        
        let program = parse_code(code);
        let mut checker = OwnershipChecker::new();
        let result = checker.check(&program);
        
        assert!(result.is_ok(), "Basic ownership transfer should work");
    }

    #[test]
    fn test_use_after_move() {
        let code = r#"
            fn main() -> i32 {
                let x = 42
                let y = <-x
                println(x)
                return 0
            }
        "#;
        
        let program = parse_code(code);
        let mut checker = OwnershipChecker::new();
        let result = checker.check(&program);
        
        assert!(result.is_err(), "Use after move should be an error");
        assert!(result.unwrap_err().contains("Use of moved variable"));
    }

    #[test]
    fn test_immutable_borrow() {
        let code = r#"
            fn main() -> i32 {
                let x = 42
                let y = &x
                println(x)
                return 0
            }
        "#;
        
        let program = parse_code(code);
        let mut checker = OwnershipChecker::new();
        let result = checker.check(&program);
        
        assert!(result.is_ok(), "Immutable borrow should work");
    }

    #[test]
    fn test_mutable_borrow() {
        let code = r#"
            fn main() -> i32 {
                let mut x = 42
                let y = &mut x
                return 0
            }
        "#;
        
        let program = parse_code(code);
        let mut checker = OwnershipChecker::new();
        let result = checker.check(&program);
        
        assert!(result.is_ok(), "Mutable borrow should work");
    }

    #[test]
    fn test_multiple_immutable_borrows() {
        let code = r#"
            fn main() -> i32 {
                let x = 42
                let y = &x
                let z = &x
                return 0
            }
        "#;
        
        let program = parse_code(code);
        let mut checker = OwnershipChecker::new();
        let result = checker.check(&program);
        
        assert!(result.is_ok(), "Multiple immutable borrows should work");
    }

    #[test]
    fn test_mutable_borrow_conflict() {
        let code = r#"
            fn main() -> i32 {
                let mut x = 42
                let y = &mut x
                let z = &mut x
                return 0
            }
        "#;
        
        let program = parse_code(code);
        let mut checker = OwnershipChecker::new();
        let result = checker.check(&program);
        
        assert!(result.is_err(), "Multiple mutable borrows should be an error");
        assert!(result.unwrap_err().contains("already borrowed"));
    }

    #[test]
    fn test_immutable_after_mutable_borrow() {
        let code = r#"
            fn main() -> i32 {
                let mut x = 42
                let y = &mut x
                let z = &x
                return 0
            }
        "#;
        
        let program = parse_code(code);
        let mut checker = OwnershipChecker::new();
        let result = checker.check(&program);
        
        assert!(result.is_err(), "Immutable borrow after mutable should be an error");
        assert!(result.unwrap_err().contains("mutably borrowed"));
    }

    #[test]
    fn test_move_borrowed_variable() {
        let code = r#"
            fn main() -> i32 {
                let x = 42
                let y = &x
                let z = <-x
                return 0
            }
        "#;
        
        let program = parse_code(code);
        let mut checker = OwnershipChecker::new();
        let result = checker.check(&program);
        
        assert!(result.is_err(), "Moving borrowed variable should be an error");
        assert!(result.unwrap_err().contains("Cannot move borrowed variable"));
    }

    #[test]
    fn test_scope_cleanup() {
        let code = r#"
            fn main() -> i32 {
                let x = 42
                {
                    let y = &x
                }
                let z = <-x
                return 0
            }
        "#;
        
        let program = parse_code(code);
        let mut checker = OwnershipChecker::new();
        let result = checker.check(&program);
        
        assert!(result.is_ok(), "Borrow should be cleaned up after scope");
    }
}
