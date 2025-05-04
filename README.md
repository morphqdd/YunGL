# Yun

**Yun** is a lightweight, dynamically typed programming language designed for simplicity and expressiveness. Built following the principles outlined in *Crafting Interpreters*, Yun provides an intuitive syntax for rapid prototyping and learning, supporting classes, inheritance, modules, and arrays.

## Features

- **Dynamic Typing**: Variables are declared without type annotations (`let x = 42`).
- **Classes and Inheritance**: Object-oriented programming with dynamic fields via `self` and automatic `init` invocation.
- **Modularity**: Import and export functionality with `use` and `export`.
- **Arrays and Loops**: Convenient array manipulation and iteration (`for`, `while`).
- **Block Scoping**: Local variables are confined to their scope.

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/yun.git
   ```
2. Ensure Rust and Cargo are installed.
3. Build the project:
   ```bash
   cd yun
   cargo build --release
   ```
4. Run an example:
   ```bash
   cargo run -- examples/blocks.yun
   ```

## Usage Examples

### Classes and Inheritance
Shows class creation and inheritance with method overriding.

```
class Person {
    init(name) {
        self.name = name;
    }

    greet() {
        print "Hello, " + self.name;
    }
}

class Worker < Person {
    init(name) {
        super.init(name);
    }

    work() {
        print self.name + " is working...";
    }

    greet() {
        super.greet();
    }
}

let worker = Worker("Gregory");
worker.greet();  // Hello, Gregory
worker.work();   // Gregory is working...
```


## How It Works

Yun is interpreted in Rust, adhering to *Crafting Interpreters* principles. The parser converts code into an AST, and the interpreter executes it, supporting dynamic typing, classes, modules, and closures.


## License

Distributed under the MIT License. See [LICENSE](LICENSE) for details.