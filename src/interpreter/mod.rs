pub mod ast;
pub mod environment;
pub mod error;
pub mod event;
pub mod exporter;
pub mod object;
pub mod parser;
pub mod render_statement;
pub mod scanner;
pub mod shell;

use crate::interpreter::ast::expr::anon_fun::AnonFun;
use crate::interpreter::ast::expr::assignment::Assign;
use crate::interpreter::ast::expr::binary::Binary;
use crate::interpreter::ast::expr::call::Call;
use crate::interpreter::ast::expr::get::{Get, GetType};
use crate::interpreter::ast::expr::grouping::Grouping;
use crate::interpreter::ast::expr::list::List;
use crate::interpreter::ast::expr::literal::Literal;
use crate::interpreter::ast::expr::logical::Logical;
use crate::interpreter::ast::expr::object::Obj;
use crate::interpreter::ast::expr::self_expr::SelfExpr;
use crate::interpreter::ast::expr::set::{Set, SetType};
use crate::interpreter::ast::expr::superclass::Super;
use crate::interpreter::ast::expr::unary::Unary;
use crate::interpreter::ast::expr::variable::Variable;
use crate::interpreter::ast::expr::{CloneExpr, Expr, ExprVisitor};
use crate::interpreter::ast::stmt::block::Block;
use crate::interpreter::ast::stmt::class::Class;
use crate::interpreter::ast::stmt::export_stmt::Export;
use crate::interpreter::ast::stmt::fun_stmt::Fun;
use crate::interpreter::ast::stmt::if_stmt::If;
use crate::interpreter::ast::stmt::let_stmt::Let;
use crate::interpreter::ast::stmt::print::Print;
use crate::interpreter::ast::stmt::return_stmt::Return;
use crate::interpreter::ast::stmt::stmt_expr::StmtExpr;
use crate::interpreter::ast::stmt::use_stmt::Use;
use crate::interpreter::ast::stmt::while_stmt::While;
use crate::interpreter::ast::stmt::{Stmt, StmtVisitor};
use crate::interpreter::environment::Environment;
use crate::interpreter::error::Result;
use crate::interpreter::error::{InterpreterError, RuntimeError, RuntimeErrorType};
use crate::interpreter::event::InterpreterEvent;
use crate::interpreter::exporter::Exporter;
use crate::interpreter::parser::Parser;
use crate::interpreter::parser::resolver::{Resolver, SomeFun};
use crate::interpreter::scanner::Scanner;
use crate::interpreter::scanner::token::Token;
use crate::interpreter::scanner::token::token_type::TokenType;
use crate::utils::next_id;
use crate::{b, rc};
use glium::winit::event_loop::EventLoopProxy;
use object::Object;
use object::callable::Callable;
use object::native_object::NativeObject;
use std::collections::HashMap;
use std::fs;
use std::ops::Deref;
use std::path::PathBuf;
use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock, mpsc};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Interpreter {
    proxy: EventLoopProxy<InterpreterEvent>,
    path: Arc<PathBuf>,
    env: Option<Arc<RwLock<Environment>>>,
    globals: Option<Arc<RwLock<Environment>>>,
    locals: Arc<RwLock<HashMap<u64, usize>>>,
    cancel_flag: Arc<AtomicBool>,
}

impl Interpreter {
    pub fn new(proxy: EventLoopProxy<InterpreterEvent>, path: PathBuf) -> Self {
        let mut globals = Environment::default();

        globals.define(
            "getWindowDimensions",
            Some(Object::Callable(Callable::build(
                next_id(),
                None,
                None,
                rc!(|interpreter, _| {
                    let (tx, rx) = mpsc::channel::<(u32, u32)>();
                    interpreter
                        .proxy
                        .send_event(InterpreterEvent::GetWindowDimensions(tx))
                        .unwrap();
                    while let Ok(dimensions) = rx.recv() {
                        return Ok(Object::Dictionary(Arc::new(RwLock::new({
                            let mut h = HashMap::new();
                            h.insert("width".into(), Object::Number(dimensions.0 as f64));
                            h.insert("height".into(), Object::Number(dimensions.1 as f64));
                            h
                        }))));
                    }
                    Ok(Object::Nil)
                }),
                rc!(|| 0),
                rc!(|| "getWindowDimensions".into()),
                false,
            ))),
        );

        globals.define(
            "tan",
            Some(Object::Callable(Callable::build(
                next_id(),
                None,
                None,
                rc!(|_, args| {
                    if let Object::Number(n) = args[0].clone() {
                        return Ok(Object::Number(n.tan()));
                    }
                    Ok(Object::Nil)
                }),
                rc!(|| 1),
                rc!(|| "tan".into()),
                false,
            ))),
        );

        globals.define(
            "sqrt",
            Some(Object::Callable(Callable::build(
                next_id(),
                None,
                None,
                rc!(|_, args| {
                    if let Object::Number(n) = args[0].clone() {
                        return Ok(Object::Number(n.sqrt()));
                    }
                    Ok(Object::Nil)
                }),
                rc!(|| 1),
                rc!(|| "sqrt".into()),
                false,
            ))),
        );

        globals.define(
            "sin",
            Some(Object::Callable(Callable::build(
                next_id(),
                None,
                None,
                rc!(|_, args| {
                    if let Object::Number(n) = args[0].clone() {
                        return Ok(Object::Number(n.sin()));
                    }
                    Ok(Object::Nil)
                }),
                rc!(|| 1),
                rc!(|| "sin".into()),
                false,
            ))),
        );

        globals.define(
            "cos",
            Some(Object::Callable(Callable::build(
                next_id(),
                None,
                None,
                rc!(|_, args| {
                    if let Object::Number(n) = args[0].clone() {
                        return Ok(Object::Number(n.cos()));
                    }
                    Ok(Object::Nil)
                }),
                rc!(|| 1),
                rc!(|| "cos".into()),
                false,
            ))),
        );

        globals.define(
            "render",
            Some(Object::Callable(Callable::build(
                next_id(),
                None,
                None,
                rc!(|interpreter, args| {
                    //println!("Call");
                    interpreter
                        .proxy
                        .send_event(InterpreterEvent::Render(args[0].clone()))?;
                    //println!("Event sent");
                    std::thread::sleep(Duration::from_millis(10));
                    Ok(Object::Nil)
                }),
                rc!(|| 1),
                rc!(|| "render".into()),
                false,
            ))),
        );

        globals.define(
            "clock",
            Some(Object::Callable(Callable::build(
                next_id(),
                None,
                None,
                rc!(|_, _| -> Result<Object> {
                    Ok(Object::Number(
                        std::time::SystemTime::now()
                            .duration_since(std::time::SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_micros() as f64,
                    ))
                }),
                rc!(|| 0),
                rc!(|| "clock".into()),
                false,
            ))),
        );

        globals.define(
            "panic",
            Some(Object::Callable(Callable::build(
                next_id(),
                None,
                None,
                rc!(|_, args| Err(RuntimeErrorType::UserPanicWithMsg(args[0].clone()).into())),
                rc!(|| 1),
                rc!(|| "panic".into()),
                false,
            ))),
        );

        globals.define(
            "string",
            Some(Object::Callable(Callable::build(
                next_id(),
                None,
                None,
                rc!(|_, args| Ok(Object::String(args[0].clone().to_string()))),
                rc!(|| 1),
                rc!(|| "string".into()),
                false,
            ))),
        );

        globals.define(
            "exit",
            Some(Object::Callable(Callable::build(
                next_id(),
                None,
                None,
                rc!(|_, _| exit(0)),
                rc!(|| 0),
                rc!(|| "exit".into()),
                false,
            ))),
        );

        globals.define(
            "exitWithCode",
            Some(Object::Callable(Callable::build(
                next_id(),
                None,
                None,
                rc!(|_, args| exit(Into::<Result<i32>>::into(args[0].clone())?)),
                rc!(|| 1),
                rc!(|| "exitWithCode".into()),
                false,
            ))),
        );

        globals.define(
            "instant",
            Some(Object::Callable(Callable::build(
                next_id(),
                None,
                None,
                rc!(|_, _| Ok(Object::NativeObject(NativeObject::new(b!(Instant::now()))))),
                rc!(|| 0),
                rc!(|| "instant".into()),
                false,
            ))),
        );

        globals.define(
            "elapsed",
            Some(Object::Callable(Callable::build(
                next_id(),
                None,
                None,
                rc!(|_, args| {
                    if let Object::NativeObject(native) = args[0].clone() {
                        if let Some(instant) = native.extract().downcast_ref::<Instant>() {
                            return Ok(Object::Number(instant.elapsed().as_micros() as f64));
                        }
                    }
                    Ok(Object::Nil)
                }),
                rc!(|| 1),
                rc!(|| "elapsed".into()),
                false,
            ))),
        );

        let globals = Arc::new(RwLock::new(globals));

        Self {
            proxy,
            path: Arc::new(path),
            env: Some(globals.clone()),
            globals: Some(globals),
            locals: Default::default(),
            cancel_flag: Arc::new(AtomicBool::new(false)),
        }
    }
    pub fn run_script(&mut self) -> Result<()> {
        let code = fs::read_to_string(self.path.as_ref()).unwrap();
        self.run(&code)?;
        Ok(())
    }

    fn run(&mut self, code: &str) -> Result<()> {
        let mut scanner = Scanner::new(code);
        let tokens = scanner.scan_tokens()?;

        let mut parser: Parser<Result<Object>> = Parser::new(tokens);
        let ast = parser.parse()?;

        let ast = Exporter::new(self.path.as_ref().clone(), ast).resolve()?;

        let mut resolver = Resolver::new(self);

        resolver.resolve(ast.iter().map(AsRef::as_ref).collect())?;

        self.interpret(ast)?;

        Ok(())
    }

    fn interpret(&mut self, statements: Vec<Box<dyn Stmt<Result<Object>>>>) -> Result<Object> {
        let mut res = Object::Void;
        for stmt in statements {
            res = self.execute(stmt.deref())?;
        }
        Ok(res)
    }

    pub fn get_cancel_flag(&self) -> Arc<AtomicBool> {
        self.cancel_flag.clone()
    }

    fn execute_block(
        &mut self,
        statements: Vec<&dyn Stmt<Result<Object>>>,
        environment: Arc<RwLock<Environment>>,
    ) -> Result<Object> {
        let previous = self.env.replace(environment);
        for stmt in statements {
            if let Err(err) = self.execute(stmt) {
                self.env.replace(previous.unwrap());
                return Err(err);
            }
        }
        self.env.replace(previous.unwrap());
        Ok(Object::Void)
    }

    #[inline]
    fn execute(&mut self, statement: &dyn Stmt<Result<Object>>) -> Result<Object> {
        if self.cancel_flag.load(Ordering::Relaxed) {
            return Ok(Object::Nil);
        }
        statement.accept(self)
    }

    #[inline]
    fn evaluate(&mut self, expr: &dyn Expr<Result<Object>>) -> Result<Object> {
        if self.cancel_flag.load(Ordering::Relaxed) {
            return Ok(Object::Nil);
        }
        expr.accept(self)
    }

    #[inline]
    pub fn resolve(&mut self, expr: Box<dyn Expr<Result<Object>>>, depth: usize) {
        self.locals.write().unwrap().insert(expr.id(), depth);
    }

    fn look_up_variable(&mut self, name: Token, var: &dyn Expr<Result<Object>>) -> Result<Object> {
        //println!("look_up_variable {}", name);
        if let Some(distance) = self.locals.read().unwrap().get(&var.id()).cloned() {
            //println!("distance: {}", distance);
            //println!("locals: {:?}", self.locals);
            Environment::get_at(self.env.clone(), distance, &name)
        } else {
            if let Some(globals) = self.globals.clone() {
                return globals.read().unwrap().get(&name);
            }
            Err(RuntimeError::new(name, RuntimeErrorType::BugEnvironmentNotInit).into())
        }
    }

    #[inline]
    fn handle_runtime_error(token: Token, res: Result<Object>) -> Result<Object> {
        if let Err(err) = res {
            if let InterpreterError::RuntimeErrorType(runtime_ty) = err {
                return match runtime_ty {
                    RuntimeErrorType::UserPanicWithMsg(msg) => {
                        Err(Interpreter::panic_handler(token, &msg.to_string()).into())
                    }
                    _ => Err(RuntimeError::new(token, runtime_ty).into()),
                };
            }
            return Err(err);
        }
        res
    }

    #[inline]
    fn panic_handler(token: Token, msg: &str) -> String {
        Interpreter::report("Panic", token.get_line(), token.get_pos_in_line(), "", msg)
    }

    fn is_truly(&self, obj: &Object) -> Result<bool> {
        Ok((!(!obj)?)? == Object::Bool(true))
    }

    pub fn error_by_token(token: Token, msg: &str) -> String {
        if token.get_type().eq(&TokenType::Eof) {
            Interpreter::report(
                "Error",
                token.get_line(),
                token.get_pos_in_line(),
                "at end",
                msg,
            )
        } else {
            Interpreter::report(
                "Error",
                token.get_line(),
                token.get_pos_in_line(),
                &format!("at '{}'", token.get_lexeme()),
                msg,
            )
        }
    }

    fn error(line: usize, pos_in_line: usize, msg: &str) -> String {
        Interpreter::report("Error", line, pos_in_line, "", msg)
    }

    fn report(report_ty: &str, line: usize, pos_in_line: usize, _where: &str, msg: &str) -> String {
        format!(
            "[{}:{}] {}{}: {}\n",
            line,
            pos_in_line,
            report_ty,
            if _where.is_empty() {
                "".to_owned()
            } else {
                " ".to_owned() + _where
            },
            msg
        )
    }
}

impl ExprVisitor<Result<Object>> for Interpreter {
    fn visit_binary(&mut self, binary: &Binary<Result<Object>>) -> Result<Object> {
        let left = self.evaluate(binary.get_left())?;
        let right = self.evaluate(binary.get_right())?;

        let res = match binary.get_op_type() {
            TokenType::EqualEqual => Ok(Object::Bool(left == right)),
            TokenType::BangEqual => Ok(Object::Bool(left != right)),
            TokenType::Greater => Ok(Object::Bool(left > right)),
            TokenType::Less => Ok(Object::Bool(left < right)),
            TokenType::GreaterEqual => Ok(Object::Bool(left >= right)),
            TokenType::LessEqual => Ok(Object::Bool(left <= right)),
            TokenType::Plus => left + right,
            TokenType::Minus => left - right,
            TokenType::Star => left * right,
            TokenType::Slash => left / right,
            _ => Err(RuntimeError::new(
                binary.get_token(),
                RuntimeErrorType::UnsupportedBinaryOperator(binary.get_op_lexeme().into()),
            )
            .into()),
        };

        Interpreter::handle_runtime_error(binary.get_token(), res)
    }

    fn visit_grouping(&mut self, grouping: &Grouping<Result<Object>>) -> Result<Object> {
        self.evaluate(grouping.get_expr())
    }

    fn visit_literal(&mut self, literal: &Literal) -> Result<Object> {
        Ok(literal.get_value().unwrap().clone())
    }

    fn visit_unary(&mut self, unary: &Unary<Result<Object>>) -> Result<Object> {
        let obj = self.evaluate(unary.get_right())?;
        let res = match unary.get_op_type() {
            TokenType::Minus => -obj,
            TokenType::Bang => !obj,
            _ => Err(RuntimeError::new(
                unary.get_token(),
                RuntimeErrorType::UnsupportedBinaryOperator(unary.get_op_lexeme().into()),
            )
            .into()),
        };

        Interpreter::handle_runtime_error(unary.get_token(), res)
    }

    fn visit_variable(&mut self, variable: &Variable) -> Result<Object> {
        self.look_up_variable(variable.get_token(), variable)
    }

    fn visit_assign(&mut self, assign: &Assign<Result<Object>>) -> Result<Object> {
        let value = self.evaluate(assign.get_value())?;
        let name = assign.get_token();
        if let Some(distance) = self
            .locals
            .read()
            .unwrap()
            .get(&assign.clone_expr().id())
            .cloned()
        {
            Environment::assign_at(self.env.clone(), distance, &name, value)
        } else {
            if let Some(globals) = self.globals.clone() {
                return globals.write().unwrap().assign(&name, value);
            }
            Err(RuntimeError::new(name, RuntimeErrorType::BugEnvironmentNotInit).into())
        }
    }

    fn visit_logical(&mut self, logical: &Logical<Result<Object>>) -> Result<Object> {
        let left = self.evaluate(logical.get_left())?;
        if logical.get_operator().get_type().eq(&TokenType::Or) {
            if self.is_truly(&left)? {
                return Ok(left);
            }
        } else if !self.is_truly(&left)? {
            return Ok(left);
        }

        self.evaluate(logical.get_right())
    }

    fn visit_call(&mut self, call_: &Call<Result<Object>>) -> Result<Object> {
        let callable = self.evaluate(call_.get_callable())?;
        let mut args: Vec<Object> = Vec::new();
        for arg in call_.get_args() {
            args.push(self.evaluate(arg)?);
        }

        let callable = match callable.clone_into_rc() {
            Object::Class(class) => Object::Callable((*class).into()),
            _ => callable,
        };

        match callable {
            Object::Callable(callable) => {
                if args.len() != callable.arity() {
                    return Err(RuntimeError::new(
                        call_.get_token(),
                        RuntimeErrorType::ArityOfFuncNotEqSizeOfArgs,
                    )
                    .into());
                }
                Interpreter::handle_runtime_error(call_.get_token(), callable.call(self, args))
            }
            _ => Err(RuntimeError::new(call_.get_token(), RuntimeErrorType::NotCallable).into()),
        }
    }

    fn visit_get(&mut self, get: &Get<Result<Object>>) -> Result<Object> {
        let (ty, obj) = get.extract();
        let obj = self.evaluate(obj)?;
        match ty {
            GetType::Name(name) => {
                match obj {
                    Object::Instance(instance) => return instance.get(name),
                    Object::Dictionary(dict) => {
                        return Ok(dict
                            .read()
                            .unwrap()
                            .get(name.get_lexeme())
                            .unwrap_or(
                                dict.read()
                                    .unwrap()
                                    .get(&format!(r#""{}""#, name.get_lexeme()))
                                    .unwrap_or(&Object::Nil),
                            )
                            .clone());
                    }
                    _ => {}
                }
                Err(
                    RuntimeError::new(name.clone(), RuntimeErrorType::OnlyInstancesHaveProperties)
                        .into(),
                )
            }
            GetType::Index(token, index) => {
                let Object::Number(index) = self.evaluate(index.clone().deref())? else {
                    return Err(
                        RuntimeError::new(token.clone(), RuntimeErrorType::MustBeANumber).into(),
                    );
                };
                let Object::List(_) = &obj else {
                    return Err(RuntimeError::new(
                        token.clone(),
                        RuntimeErrorType::OnlyListsHaveIndices,
                    )
                    .into());
                };
                Ok(obj.get_field(Object::Number(index)).unwrap())
            }
        }
        //println!("{name}");

        //println!("Eval");
    }

    fn visit_set(&mut self, set: &Set<Result<Object>>) -> Result<Object> {
        let (ty, obj, value) = set.extract();
        let obj = self.evaluate(obj)?;

        match ty {
            SetType::Name(name) => {
                match obj {
                    Object::Instance(instance) => {
                        let value = self.evaluate(value)?;
                        instance.set(name, value.clone());
                        return Ok(value);
                    }
                    Object::Dictionary(dict) => {
                        let value = self.evaluate(value)?;
                        dict.write()
                            .unwrap()
                            .insert(name.get_lexeme().into(), value.clone());
                        return Ok(value);
                    }
                    _ => {}
                }
                Err(
                    RuntimeError::new(name.clone(), RuntimeErrorType::OnlyInstancesHaveProperties)
                        .into(),
                )
            }
            SetType::Index(token, index) => {
                let Object::Number(index) = self.evaluate(index.clone().deref())? else {
                    return Err(
                        RuntimeError::new(token.clone(), RuntimeErrorType::MustBeANumber).into(),
                    );
                };
                let Object::List(list) = &obj else {
                    return Err(RuntimeError::new(
                        token.clone(),
                        RuntimeErrorType::OnlyListsHaveIndices,
                    )
                    .into());
                };
                let value = self.evaluate(value)?;
                list.write().unwrap()[index as usize] = value.clone();
                Ok(value)
            }
        }
    }

    fn visit_self(&mut self, self_val: &SelfExpr) -> Result<Object> {
        self.look_up_variable(self_val.get_name(), self_val)
    }

    fn visit_super(&mut self, super_val: &Super) -> Result<Object> {
        let (keyword, method_name) = super_val.extract();
        let distance = self
            .locals
            .read()
            .unwrap()
            .get(&<Super as Expr<Result<Object>>>::id(super_val))
            .unwrap()
            .clone();
        let superclass = Environment::get_at(self.env.clone(), distance, &keyword)?;
        let instance = Environment::get_at(
            self.env.clone(),
            1,
            &Token::builtin_void(TokenType::Slf, "self", None),
        )?;

        let method = match superclass.inner() {
            Object::Class(class) => class.find_method(method_name.get_lexeme()),
            _ => panic!("Interpreter bug!"),
        };

        if let Some(method) = method {
            if let Object::Instance(instance) = instance {
                return method.bind(instance);
            }
        }

        Err(RuntimeError::new(
            method_name.clone(),
            RuntimeErrorType::UndefinedProperty(method_name.get_lexeme().into()),
        )
        .into())
    }

    fn visit_list(&mut self, list: &List<Result<Object>>) -> Result<Object> {
        let mut values: Vec<Object> = vec![];
        for val in list.extract_values() {
            values.push(self.evaluate(val)?);
        }
        Ok(Object::List(rc!(RwLock::new(values))))
    }

    fn visit_object(&mut self, object: &Obj<Result<Object>>) -> Result<Object> {
        let values = object.extract();
        let mut obj = HashMap::new();
        for (key, value) in values.clone() {
            obj.insert(key.get_lexeme().to_string(), self.evaluate(value.deref())?);
        }
        Ok(Object::Dictionary(rc!(RwLock::new(obj))))
    }

    fn visit_anon(&mut self, anon: &AnonFun<Result<Object>>) -> Result<Object> {
        let anon = Object::function(SomeFun::AnonFun(anon.clone()), self.env.clone(), false);
        Ok(anon)
    }
}

impl StmtVisitor<Result<Object>> for Interpreter {
    fn visit_expr(&mut self, stmt: &StmtExpr<Result<Object>>) -> Result<Object> {
        self.evaluate(stmt.expr())
    }

    fn visit_print(&mut self, stmt: &Print<Result<Object>>) -> Result<Object> {
        let value = self.evaluate(stmt.expr())?;
        println!("{}", value);
        Ok(Object::Nil)
    }

    fn visit_let(&mut self, stmt: &Let<Result<Object>>) -> Result<Object> {
        match stmt.get_initializer() {
            Some(initializer) => {
                let value = self.evaluate(initializer)?;

                match &self.env {
                    None => {
                        return Err(RuntimeError::new(
                            stmt.get_ident(),
                            RuntimeErrorType::BugEnvironmentNotInit,
                        )
                        .into());
                    }
                    Some(env) => {
                        env.write()
                            .unwrap()
                            .define(stmt.get_ident().get_lexeme(), Some(value));
                    }
                }
            }
            None => match &self.env {
                None => {
                    return Err(RuntimeError::new(
                        stmt.get_ident(),
                        RuntimeErrorType::BugEnvironmentNotInit,
                    )
                    .into());
                }
                Some(env) => {
                    env.write()
                        .unwrap()
                        .define(stmt.get_ident().get_lexeme(), None);
                }
            },
        }
        Ok(Object::Nil)
    }

    fn visit_block(&mut self, stmt: &Block<Result<Object>>) -> Result<Object> {
        self.execute_block(
            stmt.get_stmts(),
            Arc::new(RwLock::new(Environment::new(self.env.clone()))),
        )?;
        Ok(Object::Nil)
    }

    fn visit_if(&mut self, stmt: &If<Result<Object>>) -> Result<Object> {
        let (cond, then, else_) = stmt.extract();
        if self.evaluate(cond)? == Object::Bool(true) {
            self.execute(then)?;
        } else if let Some(else_stmt) = else_ {
            self.execute(else_stmt)?;
        }

        Ok(Object::Nil)
    }

    fn visit_while(&mut self, stmt: &While<Result<Object>>) -> Result<Object> {
        let (cond, stmt) = stmt.extract();
        let mut evaluated_cond = self.evaluate(cond)?;
        while self.is_truly(&evaluated_cond)? {
            self.execute(stmt)?;
            evaluated_cond = self.evaluate(cond)?;
        }
        Ok(Object::Nil)
    }

    fn visit_fun(&mut self, stmt: &Fun<Result<Object>>) -> Result<Object> {
        let name = stmt.get_name();
        let func = Object::function(SomeFun::Fun(stmt.clone()), self.env.clone(), false);

        match &self.env {
            None => {
                return Err(
                    RuntimeError::new(name, RuntimeErrorType::BugEnvironmentNotInit).into(),
                );
            }
            Some(env) => {
                env.write().unwrap().define(name.get_lexeme(), Some(func));
            }
        }

        Ok(Object::Nil)
    }

    fn visit_return(&mut self, stmt: &Return<Result<Object>>) -> Result<Object> {
        let (_, expr) = stmt.extract();
        if let Some(expr) = expr {
            Err(self.evaluate(expr)?.into())
        } else {
            Err(Object::Nil.into())
        }
    }

    fn visit_class(&mut self, class: &Class<Result<Object>>) -> Result<Object> {
        let (name, methods, superclass) = class.extract();
        if let Some(env) = self.env.clone() {
            let superclass = if let Some(superclass) = superclass {
                if let Object::Arc(rc) = self.evaluate(superclass)? {
                    if let Object::Class(_) = rc.deref() {
                        Some(Object::Arc(rc))
                    } else {
                        return Err(RuntimeError::new(
                            superclass.get_token(),
                            RuntimeErrorType::SuperclassMustBeClass,
                        )
                        .into());
                    }
                } else {
                    return Err(RuntimeError::new(
                        superclass.get_token(),
                        RuntimeErrorType::SuperclassMustBeClass,
                    )
                    .into());
                }
            } else {
                None
            };

            if let Some(superclass) = superclass.clone() {
                self.env = Some(Arc::new(RwLock::new(Environment::new(self.env.clone()))));
                self.env
                    .clone()
                    .unwrap()
                    .write()
                    .unwrap()
                    .define("super", Some(superclass));
            }

            env.write().unwrap().define(name.get_lexeme(), None);

            let mut methods_ = HashMap::with_capacity(methods.len());

            for method in methods {
                let name = method.get_name();
                let func = Object::function(
                    SomeFun::Fun(method.clone()),
                    self.env.clone(),
                    method.get_name().get_lexeme().eq("init"),
                );
                methods_.insert(name.get_lexeme().to_string(), func);
            }

            let class = Object::class(name.get_lexeme(), methods_, superclass.clone());

            if superclass.is_some() {
                self.env = self.env.clone().unwrap().read().unwrap().get_enclosing();
            }

            env.write().unwrap().assign(name, Object::Arc(rc!(class)))?;
            return Ok(Object::Nil);
        }
        Err(RuntimeError::new(name.clone(), RuntimeErrorType::BugEnvironmentNotInit).into())
    }

    fn visit_export(&mut self, class: &Export<Result<Object>>) -> Result<Object> {
        let (_, stmt) = class.extract();
        self.execute(stmt)?;
        Ok(Object::Nil)
    }

    fn visit_use(&mut self, _stmt: &Use<Result<Object>>) -> Result<Object> {
        Ok(Object::Nil)
    }
}
